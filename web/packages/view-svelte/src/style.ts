export type Style = {
  [key: string]: string | number | undefined;
};

export function toStyleAttr(style: Style) {
  let s = "";
  for (const key in style) {
    const value = style[key];
    if (value === void 0) continue;
    s += `--${key}:${value};`;
  }
  return s;
}
