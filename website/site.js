const root = document.documentElement;
const themeButton = document.querySelector("[data-theme-toggle]");
const menuButton = document.querySelector("[data-menu-toggle]");

themeButton?.addEventListener("click", () => {
  const theme = root.dataset.theme === "light" ? "dark" : "light";
  root.dataset.theme = theme;
  try {
    localStorage.setItem("cdp-theme", theme);
  } catch {}
});

menuButton?.addEventListener("click", () => {
  document.body.classList.toggle("menu-open");
});

document.querySelectorAll(".main-nav a").forEach((link) => {
  link.addEventListener("click", () => document.body.classList.remove("menu-open"));
});
