import { FluentBundle, FluentResource } from "@fluent/bundle";
import { negotiateLanguages } from "@fluent/langneg";
import type { FluentVariable } from "@fluent/bundle";

interface FileBundle {
    [filename: string]: string;
}

interface LocaleBundle {
    [locale: string]: FileBundle;
}

// This is automatically populated by `tools/bundle_texts.ts` via a postbuild script
const BUNDLED_TEXTS: LocaleBundle = {
    /* %BUNDLED_TEXTS% */
};

const bundles: Record<string, FluentBundle> = {};

for (const [locale, files] of Object.entries(BUNDLED_TEXTS)) {
    const bundle = new FluentBundle(locale);
    if (files) {
        for (const [filename, text] of Object.entries(files)) {
            if (text) {
                for (const error of bundle.addResource(
                    new FluentResource(text),
                )) {
                    console.error(
                        `Error in text for ${locale} ${filename}: ${error}`,
                    );
                }
            }
        }
    }
    bundles[locale] = bundle;
}

/**
 * Gets the localised text for the given locale and text ID.
 *
 * If the locale does not contain a text for this ID, it will return null.
 *
 * @param locale Locale to prefer when retrieving text, ie "en-US"
 * @param id ID of the text to retrieve
 * @param args Any arguments to use when creating the localised text
 * @returns Localised text or null if not found
 */
function tryText(
    locale: string,
    id: string,
    args?: Record<string, FluentVariable> | null,
): string | null {
    const bundle = bundles[locale];
    if (bundle !== undefined) {
        const message = bundle.getMessage(id);
        if (message !== undefined && message.value) {
            return bundle.formatPattern(message.value, args);
        }
    }
    return null;
}

/**
 * Gets the localised text for the given text ID.
 *
 * The users preferred locales are used, in priority order, to find the given text.
 *
 * If no text is found for any preferred locale, en-US will be used.
 * If en-US does not contain a text for this ID, an error will be logged and the ID itself will be returned.
 *
 * @param id ID of the text to retrieve
 * @param args Any arguments to use when creating the localised text
 * @returns Localised text
 */
export function text(
    id: string,
    args?: Record<string, FluentVariable> | null,
): string {
    const locales = negotiateLanguages(
        navigator.languages,
        Object.keys(bundles),
        { defaultLocale: "en-US" },
    );

    for (const i in locales) {
        const result = tryText(locales[i]!, id, args);
        if (result) {
            return result;
        }
    }

    console.error(`Unknown text key '${id}'`);
    return id;
}

/**
 * Gets the localised text for the given text ID, as <p>paragraphs</p> and HTML entities safely encoded.
 *
 * The users preferred locales are used, in priority order, to find the given text.
 *
 * If no text is found for any preferred locale, en-US will be used.
 * If en-US does not contain a text for this ID, an error will be logged and the ID itself will be returned.
 *
 * @param id ID of the text to retrieve
 * @param args Any arguments to use when creating the localised text
 * @returns Localised text with each line in a Paragraph element
 */
export function textAsParagraphs(
    id: string,
    args?: Record<string, FluentVariable> | null,
): HTMLDivElement {
    const result = document.createElement("div");
    text(id, args)
        .split("\n")
        .forEach((line) => {
            const p = document.createElement("p");
            p.innerText = line;
            result.appendChild(p);
        });
    return result;
}
