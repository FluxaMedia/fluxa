class Provider {
  async data() {
    const done = storage.get("done", []);
    const items = ["Write a skin", "Share the link", "Watch it update"];
    return {
      title: "Notes (" + done.length + "/" + items.length + ")",
      items: items.map((text, index) => ({
        label: (done.includes(index) ? "[x] " : "[ ] ") + text,
        index,
      })),
    };
  }

  async toggle(args) {
    const done = new Set(storage.get("done", []));
    if (done.has(args.index)) done.delete(args.index);
    else done.add(args.index);
    storage.set("done", [...done]);
  }
}
