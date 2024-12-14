// This is automatically populated by `tools/bundle_css.ts` via a postbuild script
const CSS: string = "/* %STATIC_STYLES_CSS% */";
export function StaticStyles() {
    return <style>{CSS}</style>;
}
