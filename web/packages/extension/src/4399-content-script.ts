try {
    Object.defineProperty(window, "showBlockFlash", {
        value: () => {},
        writable: false,
    });
} catch (_e) {
    // Ignore.
}
