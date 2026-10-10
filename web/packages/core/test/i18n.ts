import { strict as assert } from "assert";
import { JSDOM } from "jsdom";
import { localizeElements } from "../src/internal/i18n";

function getText(id: string): string {
    return `<${id}>`;
}

function createRoot(html: string): ShadowRoot {
    const { document } = new JSDOM().window;
    const shadow = document.body.attachShadow({ mode: "open" });
    shadow.innerHTML = html;
    return shadow;
}

describe("localizeElements", function () {
    it("should set content of elements with data-i18n-key", function () {
        const root = createRoot(
            `<span id="a" data-i18n-key="key-a"></span>
            <div><p id="b" data-i18n-key="key-b"></p></div>`,
        );
        localizeElements(root, getText);
        assert.equal(root.getElementById("a")!.textContent, "<key-a>");
        assert.equal(root.getElementById("b")!.textContent, "<key-b>");
    });

    it("should replace existing content", function () {
        const root = createRoot(
            `<a id="a" data-i18n-key="key-a">Old <b>text</b></a>`,
        );
        localizeElements(root, getText);
        const element = root.getElementById("a")!;
        assert.equal(element.textContent, "<key-a>");
        assert.equal(element.children.length, 0);
    });

    it("should set title of elements with data-i18n-title-key", function () {
        const root = createRoot(
            `<label id="a" data-i18n-title-key="key-a">Content</label>`,
        );
        localizeElements(root, getText);
        const element = root.getElementById("a")!;
        assert.equal(element.getAttribute("title"), "<key-a>");
        assert.equal(element.textContent, "Content");
    });

    it("should set both content and title", function () {
        const root = createRoot(
            `<span id="a" data-i18n-key="key-a" data-i18n-title-key="key-b"></span>`,
        );
        localizeElements(root, getText);
        const element = root.getElementById("a")!;
        assert.equal(element.textContent, "<key-a>");
        assert.equal(element.getAttribute("title"), "<key-b>");
    });

    it("should localize SVG elements", function () {
        const root = createRoot(
            `<svg><text id="a" data-i18n-key="key-a"></text></svg>`,
        );
        localizeElements(root, getText);
        assert.equal(root.getElementById("a")!.textContent, "<key-a>");
    });

    it("should leave other elements untouched", function () {
        const root = createRoot(
            `<span id="a">Content</span>
            <span id="b" data-text="key-b" title="Title">Content</span>`,
        );
        localizeElements(root, getText);
        assert.equal(root.getElementById("a")!.textContent, "Content");
        const element = root.getElementById("b")!;
        assert.equal(element.textContent, "Content");
        assert.equal(element.getAttribute("title"), "Title");
    });
});
