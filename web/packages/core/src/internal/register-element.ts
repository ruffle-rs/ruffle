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
 * @throws Throws an error if two different elements were registered with the
 * same internal name.
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
