import { StaticStyles } from "./static-styles.js";
import { DynamicStyles } from "./dynamic-styles.js";
import { MainContainer } from "./container.js";
import { SplashScreen } from "./splash-screen.js";
import { SaveManager } from "./save-manager.js";
import { VolumeControls } from "./volume-controls.js";
import { UnsupportedVideo } from "./unsupported-video.js";
import { HardwareAcceleration } from "./hardware-acceleration.js";
import { ClipboardPermission } from "./clipboard-permission.js";
import { ContextMenuOverlay } from "./context-menu-overlay.js";

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
