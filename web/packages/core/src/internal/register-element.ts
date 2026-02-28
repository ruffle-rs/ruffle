/**
 * Number of times to try defining a custom element.
 */
const MAX_TRIES = 999;

/**
 * A mapping between internal element IDs and DOM element IDs.
 */
const privateRegistry: Record<string, Registration> = {};

interface Registration {
    class: CustomElementConstructor;
    name: string;
    internalName: string;
}

/**
 * Lookup a previously registered custom element.
 *
 * The returned object will have `name`, `class`, and `internal_name`
 * properties listing the external name, implementing class, and internal name
 * respectively.
 *
 * @param elementName The internal element name, previously used to
 * register the element with the private registry.
 * @returns The element data in the registry, or null if there is
 * no such element name registered.
 */
export function lookupElement(elementName: string): Registration | null {
    const data = privateRegistry[elementName];
    if (data !== undefined) {
        return {
            internalName: elementName,
            name: data.name,
            class: data.class,
        };
    } else {
        return null;
    }
}

/**
 * Merge two sorted arrays into another sorted array based on a comparison function.
 *
 * @param a The first sorted array.
 * @param b The second sorted array.
 * @param comparator A function returning:
 *        < 0 if x comes before y,
 *        > 0 if x comes after y,
 *        0 if equal.
 * @returns The merged, sorted array.
 */
function mergeSorted<T>(
    a: readonly T[],
    b: readonly T[],
    comparator: (x: T, y: T) => number,
): T[] {
    const result: T[] = [];

    let i = 0;
    let j = 0;

    // Merge while both arrays still have elements
    while (i < a.length && j < b.length) {
        const x = a[i]!;
        const y = b[j]!;
        if (comparator(x, y) <= 0) {
            result.push(x);
            i++;
        } else {
            result.push(y);
            j++;
        }
    }
    // Append leftovers
    while (i < a.length) {
        result.push(a[i++]!);
    }
    while (j < b.length) {
        result.push(b[j++]!);
    }

    return result;
}

/**
 * Comparison function that checks if one element comes before another in DOM order.
 *
 * @param a The first element to compare.
 * @param b The second element to compare.
 * @returns A negative number if `a` comes before `b` in DOM order,
 *          A positive number if `a` comes after `b` in DOM order,
 *          0 if they are the same node.
 */
function domComesBefore(a: Element, b: Element): number {
    if (a === b) {
        return 0;
    }

    const pos = a.compareDocumentPosition(b);

    if (pos & Node.DOCUMENT_POSITION_FOLLOWING) {
        return -1;
    }
    if (pos & Node.DOCUMENT_POSITION_PRECEDING) {
        return 1;
    }

    return 0;
}

/**
 * Get ruffle-embed selector based on tries.
 *
 * @param tries Number of tries before this custom element was defined.
 * @returns The string selector that can be used in querySelectorAll.
 */
function getSelector(tries: number): string {
    const selectors: string[] = ["ruffle-embed"];
    for (let i = 1; i <= tries; i++) {
        selectors.push(`ruffle-embed-${i}`);
    }
    return selectors.join(", ");
}

/**
 * Polyfill so document.embeds will return ruffle-embeds too.
 * A website may polyfill document.embeds if it adds custom embed-like elements itself.
 * Therefore, we fallback to the existing document.embeds in this polyfill to not conflict.
 *
 * @param tries Number of tries before this custom element was defined.
 */
function polyfillDocumentEmbeds(tries: number) {
    const orig = Object.getOwnPropertyDescriptor(Document.prototype, "embeds");
    if (!orig?.get) {
        return;
    }
    const CACHE_SYM: unique symbol = Symbol("ruffle_embeds_cache");
    interface CachedCollection extends HTMLCollection {
        [CACHE_SYM]?: true;
    }
    Object.defineProperty(Document.prototype, "embeds", {
        get(this: Document): CachedCollection {
            const documentWithCache = this as unknown as Record<
                symbol,
                CachedCollection
            >;
            const existing = documentWithCache[CACHE_SYM];
            if (existing) {
                return existing;
            }

            const nodes = (): Element[] => {
                // Fallback to existing document.embeds for non ruffle-embed elements
                const baseEmbeds = orig.get!.call(this) as HTMLCollection;
                const selector = getSelector(tries);

                const ruffleEmbeds = Array.from(
                    this.querySelectorAll(selector),
                );

                // Per https://dom.spec.whatwg.org/#interface-htmlcollection, sorted in tree order
                return mergeSorted(
                    Array.from(baseEmbeds),
                    ruffleEmbeds,
                    domComesBefore,
                );
            };

            const base = Object.create(
                HTMLCollection.prototype,
            ) as HTMLCollection;

            Object.defineProperty(base, "length", {
                enumerable: true,
                configurable: true,
                get() {
                    return nodes().length;
                },
            });

            base.item = function (index: number): Element | null {
                return nodes()[index] ?? null;
            };

            base.namedItem = function (name: string): Element | null {
                const list = nodes();
                for (const el of list) {
                    const htmlEl = el as HTMLElement;
                    if (
                        name &&
                        (htmlEl.getAttribute("name") === name ||
                            htmlEl.id === name)
                    ) {
                        return htmlEl;
                    }
                }
                return null;
            };

            (base as Iterable<Element>)[Symbol.iterator] =
                function* (): Iterator<Element> {
                    for (const el of nodes()) {
                        yield el;
                    }
                };

            const proxy = new Proxy(base, {
                get(target, prop, receiver) {
                    if (typeof prop === "string") {
                        const index = Number(prop);
                        if (!Number.isNaN(index) && index >= 0) {
                            return nodes()[index];
                        }
                    }
                    return Reflect.get(target, prop, receiver);
                },
                has(target, prop) {
                    if (typeof prop === "string") {
                        const index = Number(prop);
                        if (!Number.isNaN(index) && index >= 0) {
                            return index < nodes().length;
                        }
                    }
                    return Reflect.has(target, prop);
                },
            }) as CachedCollection;

            proxy[CACHE_SYM] = true;

            documentWithCache[CACHE_SYM] = proxy;

            return proxy;
        },
        configurable: true,
        enumerable: true,
    });
}

/**
 * Register a custom element.
 *
 * This function is designed to be tolerant of naming conflicts. If
 * registration fails, we modify the name, and try again. As a result, this
 * function returns the real element name to use.
 *
 * Calling this function multiple times will *not* register multiple elements.
 * We store a private registry mapping internal element names to DOM names.
 * Thus, the proper way to use this function is to call it every time you are
 * about to work with custom element names.
 *
 * @param elementName The internal name of the element.
 * @param elementClass The class of the element.
 *
 * You must call this function with the same class every time.
 * @returns The actual element name.
 * @throws {Error} Throws an error if two different elements were registered
 * with the same internal name.
 */
export function registerElement(
    elementName: string,
    elementClass: CustomElementConstructor,
): string {
    const registration = privateRegistry[elementName];
    if (registration !== undefined) {
        if (registration.class !== elementClass) {
            throw new Error("Internal naming conflict on " + elementName);
        } else {
            return registration.name;
        }
    }

    let tries = 0;

    if (window.customElements !== undefined) {
        while (tries < MAX_TRIES) {
            let externalName = elementName;
            if (tries > 0) {
                externalName = externalName + "-" + tries;
            }

            if (window.customElements.get(externalName) !== undefined) {
                tries += 1;
                continue;
            } else {
                window.customElements.define(externalName, elementClass);

                if (elementName === "ruffle-embed") {
                    polyfillDocumentEmbeds(tries);
                }
            }

            privateRegistry[elementName] = {
                class: elementClass,
                name: externalName,
                internalName: elementName,
            };

            return externalName;
        }
    }

    throw new Error("Failed to assign custom element " + elementName);
}
