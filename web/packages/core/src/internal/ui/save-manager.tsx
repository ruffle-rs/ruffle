/**
 * @returns The HTMLElement containing the list of save files
 */
export function SaveManager() {
    return (
        <div id="save-manager" class="modal hidden">
            <div id="modal-area" class="modal-area">
                <span class="close-modal"></span>
                <div class="general-save-options">
                    <span
                        class="modal-button"
                        data-i18n-key="save-backup-all"
                    ></span>
                </div>
                <table id="local-saves"></table>
            </div>
        </div>
    );
}
