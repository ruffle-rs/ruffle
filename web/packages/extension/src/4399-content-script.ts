try {
    Object.defineProperty(window, "showBlockFlash", {
        value: () => {},
        writable: false,
    });
} catch (e) {
    // Ignore.
}
