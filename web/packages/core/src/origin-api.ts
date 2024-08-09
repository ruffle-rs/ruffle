import { PanicAction } from "./internal/ui/panic";

/**
 * The OriginAPI is used to access behaviour that's specific to the Ruffle origin (selfhosted, demo or extension).
 */
export interface OriginAPI {
    /**
     * Display a LoadSwfError message for the given URL, customized to the Ruffle origin, if an error message specific
     * to the origin exists. Returns whether this is the case.
     * This is used to display error messages that only apply to a specific origin.
     * If false is returned, a general error message should be displayed instead, otherwise it shouldn't.
     *
     * @param swfUrl The URL for which a LoadSwfError message should be displayed.
     * @returns Whether an error message specific to the Ruffle origin exists.
     */
    loadSwfErrorMessage(swfUrl: URL | undefined):
        | {
              body: HTMLDivElement;
              actions: PanicAction[];
          }
        | undefined;

    giveTryHttpAdvice(swfUrl: URL | undefined): string;
}
