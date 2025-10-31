export class Modal {
    public static readonly MODAL_SHOW = "ruffle.modal.show";
    public static readonly MODAL_HIDE = "ruffle.modal.hide";

    private readonly modalElement: HTMLElement;
    private readonly closeButton: NodeListOf<HTMLElement>;

    constructor(target: string | HTMLElement) {
        if (target instanceof HTMLElement) {
            this.modalElement = target;
        } else {
            const el = document.querySelector<HTMLElement>(target);

            if (!el) {
                throw new Error(`Unknown modal element`);
            }

            this.modalElement = el;
        }

        this.closeButton = this.modalElement.querySelectorAll<HTMLElement>(
            "[data-ruffle-modal-dismiss]",
        );

        for (const el of this.closeButton) {
            el.addEventListener("click", this.hide.bind(this));
        }
    }

    public get element(): HTMLElement {
        return this.modalElement;
    }

    public show(): void {
        document.body.classList.add("modal-open");

        this.modalElement.style.display = "flex";

        setTimeout(() => {
            this.modalElement.classList.add("show");

            this.modalElement.dispatchEvent(
                new CustomEvent(Modal.MODAL_SHOW, {
                    bubbles: true,
                    detail: {
                        modalElement: this.modalElement,
                    },
                }),
            );
        }, 150);
    }

    public hide(): void {
        document.body.classList.remove("modal-open");

        this.modalElement.classList.remove("show");

        setTimeout((): void => {
            this.modalElement.style.display = "";

            this.modalElement.dispatchEvent(
                new CustomEvent(Modal.MODAL_HIDE, {
                    bubbles: true,
                    detail: {
                        modalElement: this.modalElement,
                    },
                }),
            );
        }, 150);
    }
}
