import React, { ReactNode, DragEvent } from "react";
import { Setup, Player as RufflePlayer, Config } from "ruffle-core";

export interface PlayerAttributes {
    id?: string | undefined;
    children?: ReactNode;
    onLoadedMetadata: (metadata: RufflePlayer.MovieMetadata) => void;
    baseConfig?: Config.BaseLoadOptions;
    onDragEnter: (event: DragEvent<HTMLElement>) => void;
    onDragLeave: (event: DragEvent<HTMLElement>) => void;
    onDragOver: (event: DragEvent<HTMLElement>) => void;
    onDragDrop: (event: DragEvent<HTMLElement>) => void;
}

export class Player extends React.Component<PlayerAttributes> {
    private readonly container: React.RefObject<HTMLDivElement>;
    private player: RufflePlayer.PlayerElement | null = null;

    // [NA] Ruffle has a bug where if you load a swf whilst it's already loading another swf, it breaks
    // Combine this with React testing everything by loading things twice to catch bugs - well, they caught the bug for sure.
    // This is a hacky workaround.
    private isLoading: boolean = false;

    constructor(props: PlayerAttributes) {
        super(props);

        this.container = React.createRef();
    }

    componentDidMount() {
        this.player = (window.RufflePlayer as Setup.PublicAPI)
            .newest()!
            .createPlayer()!;
        this.player.id = "player";
        this.player.addEventListener("loadedmetadata", () => {
            if (this.props.onLoadedMetadata) {
                this.props.onLoadedMetadata(this.player!.ruffle().metadata!);
            }
        });
        this.isLoading = false;

        // current is guaranteed to be set before this callback
        this.container.current!.appendChild(this.player);
    }

    componentWillUnmount() {
        this.player?.remove();
        this.player = null;
        this.isLoading = false;
    }

    render() {
        return (
            <div
                id={this.props.id}
                ref={this.container}
                onDragEnter={this.props.onDragEnter}
                onDragLeave={this.props.onDragLeave}
                onDragOver={this.props.onDragOver}
                onDrop={this.props.onDragDrop}
            >
                {this.props.children}
            </div>
        );
    }

    reload() {
        if (!this.isLoading) {
            this.isLoading = true;
            this.player
                ?.ruffle()
                .reload()
                .finally(() => {
                    this.isLoading = false;
                });
        }
    }

    loadUrl(url: string, options: Config.BaseLoadOptions) {
        if (!this.isLoading) {
            this.isLoading = true;
            this.player
                ?.ruffle()
                .load({ url, ...this.props.baseConfig, ...options })
                .finally(() => {
                    this.isLoading = false;
                });
        }
    }

    loadFile(file: File) {
        if (!this.isLoading) {
            this.isLoading = true;
            new Response(file)
                .arrayBuffer()
                .then((data) => {
                    return this.player?.ruffle().load({
                        data,
                        ...this.props.baseConfig,
                    });
                })
                .finally(() => {
                    this.isLoading = false;
                });
        }
    }
}
