
export interface ContextMenuInfo {
    readonly playing: boolean;
    readonly customItems: [ContextMenuItemInfo];
    readonly builtinItems: [string];
}

export interface ContextMenuItemInfo {
    readonly caption: string;
}
