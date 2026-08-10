package main

import (
	"log"
	"net/http"
	"os"
	"strconv"
	"strings"
	"time"

	"github.com/gorilla/websocket"
)

func envInt(name string, fallback int) int {
	value, err := strconv.Atoi(strings.TrimSpace(os.Getenv(name)))
	if err != nil || value <= 0 {
		return fallback
	}
	return value
}

func main() {
	port := strings.TrimSpace(os.Getenv("PORT"))
	if port == "" {
		port = "8787"
	}
	secret := os.Getenv("FLUXA_WATCH_SECRET")
	maxMembers := envInt("MAX_ROOM_MEMBERS", 12)
	ttlMinutes := envInt("ROOM_TTL_MINUTES", 360)
	h := newHub(maxMembers, time.Duration(ttlMinutes)*time.Minute)
	go h.cleanupLoop()

	upgrader := websocket.Upgrader{
		ReadBufferSize: 4096, WriteBufferSize: 4096,
		CheckOrigin: func(r *http.Request) bool { return true },
	}

	mux := http.NewServeMux()
	mux.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		_, _ = w.Write([]byte(`{"ok":true}`))
	})
	mux.HandleFunc("/ws", func(w http.ResponseWriter, r *http.Request) {
		if secret != "" && r.URL.Query().Get("token") != secret {
			http.Error(w, "unauthorized", http.StatusUnauthorized)
			return
		}
		conn, err := upgrader.Upgrade(w, r, nil)
		if err != nil {
			return
		}
		c := &client{id: newClientID(), name: "Guest", conn: conn}
		defer func() { h.leave(c); _ = conn.Close() }()

		conn.SetReadLimit(64 * 1024)
		_ = conn.SetReadDeadline(time.Now().Add(90 * time.Second))
		conn.SetPongHandler(func(string) error { return conn.SetReadDeadline(time.Now().Add(90 * time.Second)) })

		for {
			var msg clientMessage
			if err := conn.ReadJSON(&msg); err != nil {
				break
			}
			_ = conn.SetReadDeadline(time.Now().Add(90 * time.Second))
			switch strings.ToLower(strings.TrimSpace(msg.Type)) {
			case msgCreate:
				c.name = sanitizeName(msg.Name)
				r, err := h.createRoom(c)
				if err != nil {
					_ = c.writeJSON(map[string]any{"type": msgError, "message": err.Error()})
					continue
				}
				_ = c.writeJSON(h.roomPayload(c, r))
				h.broadcastMembers(r)
			case msgJoin:
				c.name = sanitizeName(msg.Name)
				r, err := h.joinRoom(c, msg.Room)
				if err != nil {
					_ = c.writeJSON(map[string]any{"type": msgError, "message": err.Error()})
					continue
				}
				_ = c.writeJSON(h.roomPayload(c, r))
				h.broadcastMembers(r)
				h.sendCurrentState(c, r)
			case msgLeave:
				h.leave(c)
			case msgState:
				h.handleState(c, msg)
			case msgContent:
				h.handleContent(c, msg)
			case msgBuffering:
				h.handleBuffering(c, msg.Buffering)
			case msgPing:
				_ = c.writeJSON(map[string]any{"type": msgPong, "serverTimeMs": time.Now().UnixMilli(), "clientTimeMs": msg.ClientTimeMs})
			default:
				_ = c.writeJSON(map[string]any{"type": msgError, "message": "unknown message type"})
			}
		}
	})

	server := &http.Server{
		Addr:              ":" + port,
		Handler:           mux,
		ReadHeaderTimeout: 10 * time.Second,
		IdleTimeout:       120 * time.Second,
	}
	log.Printf("Fluxa Watch Together server listening on :%s (max room members=%d)", port, maxMembers)
	if err := server.ListenAndServe(); err != nil && err != http.ErrServerClosed {
		log.Fatal(err)
	}
}
