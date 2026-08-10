package main

import (
	"crypto/rand"
	"strconv"
	"strings"
	"sync/atomic"
)

const roomAlphabet = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789"
const roomCodeLength = 6

const (
	msgCreate    = "create"
	msgJoin      = "join"
	msgLeave     = "leave"
	msgState     = "state"
	msgContent   = "content"
	msgBuffering = "buffering"
	msgPing      = "ping"
	msgPong      = "pong"
	msgRoom      = "room"
	msgMembers   = "members"
	msgSync      = "sync"
	msgError     = "error"
)

type clientMessage struct {
	Type         string `json:"type"`
	Name         string `json:"name,omitempty"`
	Room         string `json:"room,omitempty"`
	PositionMs   int64  `json:"positionMs,omitempty"`
	DurationMs   int64  `json:"durationMs,omitempty"`
	Playing      bool   `json:"playing,omitempty"`
	Buffering    bool   `json:"buffering,omitempty"`
	ContentID    string `json:"contentId,omitempty"`
	ContentType  string `json:"contentType,omitempty"`
	VideoID      string `json:"videoId,omitempty"`
	Title        string `json:"title,omitempty"`
	ClientTimeMs int64  `json:"clientTimeMs,omitempty"`
}

type memberView struct {
	ID        string `json:"id"`
	Name      string `json:"name"`
	Buffering bool   `json:"buffering"`
}

type playbackState struct {
	PositionMs  int64
	DurationMs  int64
	Playing     bool
	Buffering   bool
	ContentID   string
	ContentType string
	VideoID     string
	Title       string
}

func normalizeRoomCode(value string) string {
	return strings.ToUpper(strings.TrimSpace(value))
}

func validRoomCode(value string) bool {
	if len(value) != roomCodeLength {
		return false
	}
	for _, ch := range value {
		if !strings.ContainsRune(roomAlphabet, ch) {
			return false
		}
	}
	return true
}

func sanitizeName(name string) string {
	name = strings.TrimSpace(name)
	if name == "" {
		return "Guest"
	}
	runes := []rune(name)
	if len(runes) > 32 {
		runes = runes[:32]
	}
	return string(runes)
}

func randomCode(length int) (string, error) {
	out := make([]byte, length)
	raw := make([]byte, length)
	if _, err := rand.Read(raw); err != nil {
		return "", err
	}
	for i := range out {
		out[i] = roomAlphabet[int(raw[i])%len(roomAlphabet)]
	}
	return string(out), nil
}

var clientCounter uint64

func newClientID() string {
	n := atomic.AddUint64(&clientCounter, 1)
	suffix, _ := randomCode(4)
	return strconv.FormatUint(n, 36) + suffix
}
