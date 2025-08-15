import { StaticStyles } from "./static-styles";
import { DynamicStyles } from "./dynamic-styles";
import { MainContainer } from "./container";
import { SplashScreen } from "./splash-screen";
import { SaveManager } from "./save-manager";
import { VolumeControls } from "./volume-controls";
import { UnsupportedVideo } from "./unsupported-video";
import { HardwareAcceleration } from "./hardware-acceleration";
import { ClipboardPermission } from "./clipboard-permission";
import { ContextMenuOverlay } from "./context-menu-overlay";

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
