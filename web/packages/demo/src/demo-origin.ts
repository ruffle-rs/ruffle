import { OriginAPI } from "ruffle-core/dist/origin-api";
import { PanicAction } from "ruffle-core/dist/internal/ui/panic";

export class DemoOrigin implements OriginAPI {
    loadSwfErrorMessage(_swfUrl: URL | undefined):
        | {
              body: HTMLDivElement;
              actions: PanicAction[];
          }
        | undefined {
        return undefined;
    }

    giveTryHttpAdvice(_swfUrl: URL | undefined): string {
        return "false";
    }
}
