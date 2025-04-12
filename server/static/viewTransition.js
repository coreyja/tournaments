window.addEventListener("pageswap", (event) => {
  // Save scroll position to sessionStorage before the transition
  sessionStorage.setItem("scrollPosition", window.scrollY.toString());
  console.log("pageswap");
});

// On the loaded page after navigation
window.addEventListener("pagereveal", (event) => {
  // Restore scroll position before new page renders
  const savedPosition = sessionStorage.getItem("scrollPosition");

  const fromURL = new URL(navigation.activation.from.url);
  const currentURL = new URL(navigation.activation.entry.url);
  if (savedPosition && fromURL.pathname === currentURL.pathname) {
    window.scrollTo(0, parseInt(savedPosition));
  }
  console.log("pagereveal");
});
