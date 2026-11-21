class Provider {
  async data() {
    const week = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
    const days = week.slice(0, Number(settings.days));
    const done = storage.get("done", []);
    return {
      title: settings.title,
      nowPlaying: state("player.title") || "",
      quiet: settings.quiet,
      items: days.map((day, index) => ({
        day,
        label: day + " planning",
        done: done.includes(index),
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

  async onSetting(key, value) {
    if (key === "quiet" && value) app.call("player.volume", { value: settings.quietVolume });
  }
}
