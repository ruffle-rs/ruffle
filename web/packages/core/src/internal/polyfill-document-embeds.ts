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
export function mergeSorted<T>(
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
export function polyfillDocumentEmbeds(tries: number) {
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

            let cachedNodes: Element[] | null = null;

            const getFreshNodes = (): Element[] => {
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

            const nodes = (): Element[] => {
                if (cachedNodes !== null) {
                    return cachedNodes;
                }
                cachedNodes = getFreshNodes();
                // Clear the cache at the end of the current microtask/execution cycle
                queueMicrotask(() => {
                    cachedNodes = null;
                });
                return cachedNodes;
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

                        // Let native properties ('length', 'item', etc.) pass through normally
                        if (Reflect.has(target, prop)) {
                            return Reflect.get(target, prop, receiver);
                        }

                        // Fallback to named item resolution for standard dot/bracket notation
                        const element = target.namedItem(prop);
                        if (element) {
                            return element;
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

                        // Check if it's a native property or exists in the named list
                        if (Reflect.has(target, prop)) {
                            return true;
                        }
                        return target.namedItem(prop) !== null;
                    }
                    return Reflect.has(target, prop);
                },
                ownKeys() {
                    const len = nodes().length;
                    const keys: string[] = [];
                    for (let i = 0; i < len; i++) {
                        keys.push(String(i));
                    }
                    return keys;
                },
                getOwnPropertyDescriptor(target, prop) {
                    if (typeof prop === "string") {
                        const index = Number(prop);
                        if (
                            !Number.isNaN(index) &&
                            index >= 0 &&
                            index < nodes().length
                        ) {
                            return {
                                enumerable: true,
                                configurable: true,
                                writable: false,
                                value: nodes()[index],
                            };
                        }
                    }
                    return Reflect.getOwnPropertyDescriptor(target, prop);
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
