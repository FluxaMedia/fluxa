class Provider {
  async data() {
    const days = ["Mon", "Tue", "Wed", "Thu", "Fri"];
    const done = SCRAPER_SETTINGS.done || [];
    return {
      title: "This week",
      items: days.map((day, index) => ({
        day,
        label: day + " planning",
        done: done.includes(index),
        index,
      })),
    };
  }

  async toggle(args) {
    const done = new Set(SCRAPER_SETTINGS.done || []);
    if (done.has(args.index)) done.delete(args.index);
    else done.add(args.index);
    return { state: { done: [...done] } };
  }
}
