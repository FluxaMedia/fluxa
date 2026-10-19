export function mountArtwork(app, below, above) {
  const pools = { below: [], above: [] };

  const place = (img, item) => {
    const [x, y, w, h] = item.rect;
    const [cx, cy, cw, ch] = item.clip;
    const style = img.style;
    style.left = `${x}px`;
    style.top = `${y}px`;
    style.width = `${w}px`;
    style.height = `${h}px`;
    style.borderRadius = `${item.radius}px`;
    style.objectFit = item.contain ? "contain" : "cover";
    style.objectPosition = `${item.anchor[0] * 100}% ${item.anchor[1] * 100}%`;
    style.background = item.contain ? "transparent" : "";
    const top = Math.max(cy - y, 0);
    const left = Math.max(cx - x, 0);
    const right = Math.max(x + w - (cx + cw), 0);
    const bottom = Math.max(y + h - (cy + ch), 0);
    style.clipPath = `inset(${top}px ${right}px ${bottom}px ${left}px round ${item.radius}px)`;
  };

  const sync = (pool, layer, items) => {
    while (pool.length < items.length) {
      const img = document.createElement("img");
      img.alt = "";
      img.decoding = "async";
      img.referrerPolicy = "no-referrer";
      img.onerror = () => (img.style.visibility = "hidden");
      img.onload = () => (img.style.visibility = "visible");
      layer.append(img);
      pool.push(img);
    }
    pool.forEach((img, index) => {
      const item = items[index];
      if (!item) {
        img.style.display = "none";
        return;
      }
      img.style.display = "block";
      if (img.dataset.src !== item.url) {
        img.dataset.src = item.url;
        img.src = item.url;
      }
      place(img, item);
    });
  };

  return () => {
    const json = app.take_dom_images();
    if (json === undefined || json === null) return;
    const items = JSON.parse(json);
    sync(pools.below, below, items.filter((item) => !item.contain));
    sync(pools.above, above, items.filter((item) => item.contain));
  };
}
