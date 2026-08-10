package main

import (
	"testing"
	"time"
)

func TestRoomCodeValidation(t *testing.T) {
	if !validRoomCode("ABC234") {
		t.Fatal("expected generated-style room code to be valid")
	}
	if validRoomCode("ABCDEF1") {
		t.Fatal("expected wrong length to be invalid")
	}
	if validRoomCode("ABC10O") {
		t.Fatal("expected ambiguous characters to be invalid")
	}
}

func TestHostMigrationIsDeterministic(t *testing.T) {
	h := newHub(12, time.Hour)
	host := &client{id: "host", name: "Host"}
	guestB := &client{id: "b", name: "B"}
	guestA := &client{id: "a", name: "A"}
	r := &room{code: "ABC234", hostID: host.id, clients: map[string]*client{}, updatedAt: time.Now()}
	r.clients[host.id], r.clients[guestB.id], r.clients[guestA.id] = host, guestB, guestA
	host.room, guestB.room, guestA.room = r, r, r
	h.rooms[r.code] = r

	h.mu.Lock()
	change := h.leaveLocked(host)
	h.mu.Unlock()

	if change == nil {
		t.Fatal("expected remaining room change")
	}
	if r.hostID != "a" {
		t.Fatalf("expected stable lexical host promotion, got %q", r.hostID)
	}
}

func TestExpiredRoomsAreDetachedUnderLockAndReturnedForIO(t *testing.T) {
	h := newHub(12, time.Minute)
	c := &client{id: "c"}
	r := &room{code: "ABC234", hostID: c.id, clients: map[string]*client{c.id: c}, updatedAt: time.Now().Add(-2 * time.Minute)}
	c.room = r
	h.rooms[r.code] = r

	expired := h.collectExpiredRooms(time.Now().Add(-time.Minute))
	if len(expired) != 1 || len(expired[0]) != 1 {
		t.Fatalf("unexpected expired snapshot: %#v", expired)
	}
	if c.room != nil {
		t.Fatal("client should be detached before network IO")
	}
	if h.rooms[r.code] != nil {
		t.Fatal("expired room should be removed")
	}
}
