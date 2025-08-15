/**
 * @returns The HTMLElement representing the context menu
 */
export function ContextMenuOverlay() {
    return (
        <div id="context-menu-overlay" class="hidden">
            <ul id="context-menu"></ul>
        </div>
    );
}
