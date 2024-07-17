import { StaticStyles } from "./internal/ui/static-styles";
import { DynamicStyles } from "./internal/ui/dynamic-styles";
import { MainContainer } from "./internal/ui/container";
import { SplashScreen } from "./internal/ui/splash-screen";
import { SaveManager } from "./internal/ui/save-manager";
import { VolumeControls } from "./internal/ui/volume-controls";
import { UnsupportedVideo } from "./internal/ui/unsupported-video";
import { HardwareAcceleration } from "./internal/ui/hardware-acceleration";
import { ClipboardPermission } from "./internal/ui/clipboard-permission";
import { ContextMenuOverlay } from "./internal/ui/context-menu-overlay";

/*
 *
 * The shadow template which is used to fill the actual Ruffle player element
 * on the page.
 *
 */

export const ruffleShadowTemplate = document.createElement("template");
ruffleShadowTemplate.content.appendChild(<StaticStyles />);
ruffleShadowTemplate.content.appendChild(<DynamicStyles />);
ruffleShadowTemplate.content.appendChild(<MainContainer />);
ruffleShadowTemplate.content.appendChild(<SplashScreen />);
ruffleShadowTemplate.content.appendChild(<SaveManager />);
ruffleShadowTemplate.content.appendChild(<VolumeControls />);
ruffleShadowTemplate.content.appendChild(<UnsupportedVideo />);
ruffleShadowTemplate.content.appendChild(<HardwareAcceleration />);
ruffleShadowTemplate.content.appendChild(<ClipboardPermission />);
ruffleShadowTemplate.content.appendChild(<ContextMenuOverlay />);