import { OriginAPI } from "ruffle-core/dist/origin-api";
import { textAsParagraphs } from "ruffle-core/dist/internal/i18n";
import { SUPPORTED_PROTOCOLS } from "ruffle-core/dist/internal/constants";
import { CommonActions, PanicAction } from "ruffle-core/dist/internal/ui/panic";

export class ExtensionOrigin implements OriginAPI {
    private inExtensionPlayer: boolean;

    constructor() {
        this.inExtensionPlayer = false;
    }

    setInExtensionPlayer() {
        this.inExtensionPlayer = true;
    }

    loadSwfErrorMessage(swfUrl: URL | undefined):
        | {
              body: HTMLDivElement;
              actions: PanicAction[];
          }
        | undefined {
        if (!this.inExtensionPlayer || swfUrl === undefined) {
            return undefined;
        }

        const urlExtensionProtocol = document.baseURI.split(":")[0] + ":";
        if (swfUrl.protocol === urlExtensionProtocol) {
            // The user entered an invalid URL
            const urlExtensionPart = document.baseURI.split("player.html")[0]!;
            return {
                body: textAsParagraphs("error-no-valid-url", {
                    url: swfUrl.href.replace(urlExtensionPart, ""),
                }),
                actions: [CommonActions.ShowDetails],
            };
        } else if (swfUrl.protocol === "file:") {
            // The user entered a local file URL
            return {
                body: textAsParagraphs("error-local-root-url"),
                actions: [CommonActions.ShowDetails],
            };
        } else if (!SUPPORTED_PROTOCOLS.includes(swfUrl.protocol)) {
            // The user entered a URL with an unsupported protocol
            return {
                body: textAsParagraphs("error-unsupported-root-protocol", {
                    protocol: swfUrl.protocol,
                }),
                actions: [CommonActions.ShowDetails],
            };
        }

        return undefined;
    }

    giveTryHttpAdvice(swfUrl: URL | undefined): string {
        return this.inExtensionPlayer
            ? (swfUrl?.protocol?.includes("https")?.toString() ?? "true")
            : "false";
    }
}
