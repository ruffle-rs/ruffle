import { strict as assert } from "assert";
import { JSDOM } from "jsdom";
import { localizeElements } from "../src/internal/i18n";

function getText(id: string): string {
    return `<${id}>`;
}

function createRoot(createContent: (document: Document) => Node[]): ShadowRoot {
    const { document } = new JSDOM().window;
    const shadow = document.body.attachShadow({ mode: "open" });
    shadow.append(...createContent(document));
    return shadow;
}

function createElement<K extends keyof HTMLElementTagNameMap>(
    document: Document,
    tagName: K,
    attributes: Record<string, string> = {},
    ...children: (Node | string)[]
): HTMLElementTagNameMap[K] {
    const element = document.createElement(tagName);

    for (const [name, value] of Object.entries(attributes)) {
        element.setAttribute(name, value);
    }

    element.append(...children);
    return element;
}

describe("localizeElements", function () {
    it("should set content of elements with data-i18n-key", function () {
        const root = createRoot((document) => [
            createElement(document, "span", {
                id: "a",
                "data-i18n-key": "key-a",
            }),
            createElement(
                document,
                "div",
                {},
                createElement(document, "p", {
                    id: "b",
                    "data-i18n-key": "key-b",
                }),
            ),
        ]);

        localizeElements(root, getText);
        assert.equal(root.getElementById("a")!.textContent, "<key-a>");
        assert.equal(root.getElementById("b")!.textContent, "<key-b>");
    });

    it("should replace existing content", function () {
        const root = createRoot((document) => [
            createElement(
                document,
                "a",
                { id: "a", "data-i18n-key": "key-a" },
                "Old ",
                createElement(document, "b", {}, "text"),
            ),
        ]);

        localizeElements(root, getText);
        const element = root.getElementById("a")!;
        assert.equal(element.textContent, "<key-a>");
        assert.equal(element.children.length, 0);
    });

    it("should set title of elements with data-i18n-title-key", function () {
        const root = createRoot((document) => [
            createElement(
                document,
                "label",
                { id: "a", "data-i18n-title-key": "key-a" },
                "Content",
            ),
        ]);

        localizeElements(root, getText);
        const element = root.getElementById("a")!;
        assert.equal(element.getAttribute("title"), "<key-a>");
        assert.equal(element.textContent, "Content");
    });

    it("should set both content and title", function () {
        const root = createRoot((document) => [
            createElement(document, "span", {
                id: "a",
                "data-i18n-key": "key-a",
                "data-i18n-title-key": "key-b",
            }),
        ]);

        localizeElements(root, getText);
        const element = root.getElementById("a")!;
        assert.equal(element.textContent, "<key-a>");
        assert.equal(element.getAttribute("title"), "<key-b>");
    });

    it("should localize SVG elements", function () {
        const root = createRoot((document) => {
            const namespace = "http://www.w3.org/2000/svg";
            const svg = document.createElementNS(namespace, "svg");
            const text = document.createElementNS(namespace, "text");

            text.id = "a";
            text.setAttribute("data-i18n-key", "key-a");
            svg.append(text);

            return [svg];
        });

        localizeElements(root, getText);
        assert.equal(root.getElementById("a")!.textContent, "<key-a>");
    });

    it("should leave other elements untouched", function () {
        const root = createRoot((document) => [
            createElement(document, "span", { id: "a" }, "Content"),
            createElement(
                document,
                "span",
                { id: "b", "data-text": "key-b", title: "Title" },
                "Content",
            ),
        ]);

        localizeElements(root, getText);
        assert.equal(root.getElementById("a")!.textContent, "Content");
        const element = root.getElementById("b")!;
        assert.equal(element.textContent, "Content");
        assert.equal(element.getAttribute("title"), "Title");
    });
});
