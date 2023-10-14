import { BaseLoadOptions } from "ruffle-core";

export interface LoadMessage {
    type: "load";
    config: BaseLoadOptions;
}

export interface PingMessage {
    type: "ping";
}

export interface OpenURLMessage {
    type: "open_url_in_player";
    url: string;
}

export type Message = LoadMessage | PingMessage | OpenURLMessage;

export function isMessage(object: unknown): object is Message {
    return (object as Message).type !== undefined;
}
