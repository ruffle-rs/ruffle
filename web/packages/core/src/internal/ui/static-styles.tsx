// This is automatically populated by `tools/bundle_css.ts` via a postbuild script
const CSS: string = "/* %STATIC_STYLES_CSS% */";
/**
 * @returns The HTMLElement containing the static styles for the Ruffle elements
 */
export function StaticStyles() {
    return <style>{CSS}</style>;
}
