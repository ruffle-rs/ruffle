/**
 * A mapping between internal element IDs and DOM element IDs.
 */
const privateRegistry: Record<string, Registration> = {};

interface Registration {
    class: CustomElementConstructor;
    name: string;
    internal_name: string;
}

/**
 * Lookup a previously registered custom element.
 *
 * @param {string} element_name The internal element name, previously used to
 * register the element with the private registry.
 *
 * @returns {object|null} The element data in the registry, or null if there is
 * no such element name registered.
 *
 * The returned object will have `name`, `class`, and `internal_name`
 * properties listing the external name, implementing class, and internal name
 * respectively.
 */
export function lookupElement(element_name: string) {
    const data = privateRegistry[element_name];
    if (data !== undefined) {
        return {
            internal_name: element_name,
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
 * @param {string} element_name The internal name of the element.
 * @param {CustomElementConstructor} element_class The class of the element.
 *
 * You must call this function with the same class every time.
 *
 * @returns {string} The actual element name.
 * @throws Throws an error if two different elements were registered with the
 * same internal name.
 */
export function register_element(
    element_name: string,
    element_class: CustomElementConstructor
) {
    if (privateRegistry[element_name] !== undefined) {
        if (privateRegistry[element_name].class !== element_class) {
            throw new Error("Internal naming conflict on " + element_name);
        } else {
            return privateRegistry[element_name].name;
        }
    }

    let tries = 0;

    while (true) {
        try {
            let external_name = element_name;
            if (tries > 0) {
                external_name = external_name + "-" + tries;
            }

            window.customElements.define(external_name, element_class);
            privateRegistry[element_name] = {
                class: element_class,
                name: external_name,
                internal_name: element_name,
            };

            return external_name;
        } catch (e) {
            if (e.name === "NotSupportedError") {
                tries += 1;
            }
        }
    }
}
