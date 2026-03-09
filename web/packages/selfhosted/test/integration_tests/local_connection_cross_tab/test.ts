import {
    injectRuffleAndWait,
    expectTraceOutput,
    setupAndPlay,
    waitForPlayerToLoad,
} from "../../utils.js";
import { expect } from "chai";
import { Player, Setup } from "ruffle-core";

// Helpers to interact with the LocalConnection SWFs via ExternalInterface.

async function createPlayer(
    browser: WebdriverIO.Browser,
    swfPath: string,
): Promise<ChainablePromiseElement> {
    const player = await browser.execute((swfPath) => {
        const ruffle = (window.RufflePlayer as Setup.PublicAPI).newest();
        const player = ruffle!.createPlayer();
        player.style.width = "100px";
        player.style.height = "100px";
        const container = document.getElementById("test-container");
        container!.appendChild(player);
        void player.ruffle().load({
            url: swfPath,
            allowScriptAccess: true,
        });
        return player;
    }, swfPath);
    await waitForPlayerToLoad(browser, player);
    await setupAndPlay(browser, player);
    // Consume the initial trace to ensure the SWF is fully loaded and log is clean
    await expectTraceOutput(browser, player, ["Hello from Flash!"]);
    return player;
}

async function callExternalInterface(
    browser: WebdriverIO.Browser,
    player: ChainablePromiseElement,
    methodName: string,
    ...args: unknown[]
): Promise<unknown> {
    const argsJson = JSON.stringify(args);
    return await browser.execute(
        (playerElement, methodName, argsJson) => {
            const name = methodName as unknown as string;
            const parsedArgs = JSON.parse(
                argsJson as unknown as string,
            ) as unknown[];
            return (playerElement as Player.PlayerElement)
                .ruffle()
                .callExternalInterface(name, ...parsedArgs);
        },
        await player,
        methodName,
        argsJson,
    );
}

async function connectReceiver(
    browser: WebdriverIO.Browser,
    player: ChainablePromiseElement,
    channelName: string,
): Promise<string> {
    return (await callExternalInterface(
        browser,
        player,
        "connectLC",
        channelName,
    )) as string;
}

async function disconnectReceiver(
    browser: WebdriverIO.Browser,
    player: ChainablePromiseElement,
): Promise<void> {
    await callExternalInterface(browser, player, "disconnectLC");
}

async function sendMessage(
    browser: WebdriverIO.Browser,
    player: ChainablePromiseElement,
    connectionName: string,
    methodName: string,
    ...args: unknown[]
): Promise<void> {
    await callExternalInterface(
        browser,
        player,
        "sendLC",
        connectionName,
        methodName,
        ...args,
    );
}

async function connectSender(
    browser: WebdriverIO.Browser,
    player: ChainablePromiseElement,
    channelName: string,
): Promise<string> {
    return (await callExternalInterface(
        browser,
        player,
        "connectLC",
        channelName,
    )) as string;
}

// Switch to a tab by its handle.
async function switchToTab(browser: WebdriverIO.Browser, handle: string) {
    await browser.switchToWindow(handle);
}

// Clean up localStorage keys from our LocalConnection implementation.
async function cleanupLocalStorage(browser: WebdriverIO.Browser) {
    await browser.execute(() => {
        const storage = window.localStorage;
        const toRemove: string[] = [];
        for (let i = 0; i < storage.length; i++) {
            const key = storage.key(i);
            if (key && key.startsWith("__ruffle_lc:")) {
                toRemove.push(key);
            }
        }
        for (const key of toRemove) {
            storage.removeItem(key);
        }
    });
}

describe("LocalConnection cross-tab", () => {
    let tabA: string;
    let tabB: string;
    let receiverPlayer: ChainablePromiseElement;
    let senderPlayer: ChainablePromiseElement;

    before("Set up two tabs", async () => {
        // Tab A: receiver
        await browser.url("http://localhost:4567/test_assets/js_api.html");
        await injectRuffleAndWait(browser);

        // Clean any stale localStorage entries from previous test runs
        await cleanupLocalStorage(browser);

        receiverPlayer = await createPlayer(
            browser,
            "/test/integration_tests/local_connection_cross_tab/receiver.swf",
        );

        const handles = await browser.getWindowHandles();
        tabA = handles[0]!;

        // Tab B: sender in new window
        await browser.newWindow(
            "http://localhost:4567/test_assets/js_api.html",
        );
        await injectRuffleAndWait(browser);
        senderPlayer = await createPlayer(
            browser,
            "/test/integration_tests/local_connection_cross_tab/sender.swf",
        );

        const handles2 = await browser.getWindowHandles();
        tabB = handles2.find((h: string) => h !== tabA)!;
    });

    after("Clean up", async () => {
        // Clean up localStorage
        await cleanupLocalStorage(browser);

        // Close second tab
        const handles = await browser.getWindowHandles();
        if (handles.length > 1) {
            await switchToTab(browser, tabB);
            await browser.closeWindow();
            await switchToTab(browser, tabA);
        }
    });

    it("receiver connects to a channel", async () => {
        await switchToTab(browser, tabA);
        const result = await connectReceiver(
            browser,
            receiverPlayer,
            "testChannel",
        );
        expect(result).to.equal("ok");
        await expectTraceOutput(browser, receiverPlayer, [
            "connected:testChannel",
        ]);
    });

    it("cross-tab send delivers message to receiver", async () => {
        // Sender sends from Tab B
        await switchToTab(browser, tabB);
        await sendMessage(browser, senderPlayer, "testChannel", "test");
        await expectTraceOutput(browser, senderPlayer, [
            "sent:testChannel:test",
        ]);

        // Wait for status event on sender
        await expectTraceOutput(browser, senderPlayer, ["status:status"]);

        // Check receiver got the message in Tab A
        await switchToTab(browser, tabA);
        // Give a moment for the BroadcastChannel message to arrive and be processed
        await browser.pause(500);
        await expectTraceOutput(browser, receiverPlayer, [
            "received:test:0 args",
        ]);
    });

    it("cross-tab send with arguments", async () => {
        await switchToTab(browser, tabB);
        await sendMessage(
            browser,
            senderPlayer,
            "testChannel",
            "test",
            42,
            "hello",
            true,
        );
        await expectTraceOutput(browser, senderPlayer, [
            "sent:testChannel:test",
        ]);
        await expectTraceOutput(browser, senderPlayer, ["status:status"]);

        await switchToTab(browser, tabA);
        await browser.pause(500);
        await expectTraceOutput(browser, receiverPlayer, [
            "received:test:3 args",
            "  args=42,hello,true",
        ]);
    });

    it("duplicate connect fails across tabs", async () => {
        // Sender in Tab B tries to connect to same name
        await switchToTab(browser, tabB);
        const result = await connectSender(
            browser,
            senderPlayer,
            "testChannel",
        );
        expect(result).to.equal("error");
        await expectTraceOutput(browser, senderPlayer, [
            "connect_failed:testChannel",
        ]);
    });

    it("send to non-existent channel gets error status", async () => {
        await switchToTab(browser, tabB);
        await sendMessage(browser, senderPlayer, "nonExistentChannel", "test");
        await expectTraceOutput(browser, senderPlayer, [
            "sent:nonExistentChannel:test",
        ]);
        await expectTraceOutput(browser, senderPlayer, ["status:error"]);
    });

    it("case insensitive channel names", async () => {
        await switchToTab(browser, tabB);
        await sendMessage(browser, senderPlayer, "TESTCHANNEL", "test");
        await expectTraceOutput(browser, senderPlayer, [
            "sent:TESTCHANNEL:test",
        ]);
        await expectTraceOutput(browser, senderPlayer, ["status:status"]);

        await switchToTab(browser, tabA);
        await browser.pause(500);
        await expectTraceOutput(browser, receiverPlayer, [
            "received:test:0 args",
        ]);
    });

    it("send after disconnect gets error status", async () => {
        // Disconnect receiver in Tab A
        await switchToTab(browser, tabA);
        await disconnectReceiver(browser, receiverPlayer);
        await expectTraceOutput(browser, receiverPlayer, ["disconnected"]);

        // Send from Tab B - should fail now
        await switchToTab(browser, tabB);
        await sendMessage(browser, senderPlayer, "testChannel", "test");
        await expectTraceOutput(browser, senderPlayer, [
            "sent:testChannel:test",
        ]);
        await expectTraceOutput(browser, senderPlayer, ["status:error"]);
    });

    it("connect succeeds after remote disconnect", async () => {
        // Now sender should be able to connect to the freed name
        await switchToTab(browser, tabB);
        const result = await connectSender(
            browser,
            senderPlayer,
            "testChannel",
        );
        expect(result).to.equal("ok");
        await expectTraceOutput(browser, senderPlayer, [
            "connected:testChannel",
        ]);

        // Clean up: disconnect sender
        await callExternalInterface(browser, senderPlayer, "disconnectLC");
        await expectTraceOutput(browser, senderPlayer, ["disconnected"]);
    });

    it("underscore-prefixed names work cross-tab", async () => {
        // Connect receiver with underscore name
        await switchToTab(browser, tabA);
        const result = await connectReceiver(
            browser,
            receiverPlayer,
            "_globalChannel",
        );
        expect(result).to.equal("ok");
        await expectTraceOutput(browser, receiverPlayer, [
            "connected:_globalChannel",
        ]);

        // Send from Tab B
        await switchToTab(browser, tabB);
        await sendMessage(browser, senderPlayer, "_globalChannel", "test");
        await expectTraceOutput(browser, senderPlayer, [
            "sent:_globalChannel:test",
        ]);
        await expectTraceOutput(browser, senderPlayer, ["status:status"]);

        // Verify received
        await switchToTab(browser, tabA);
        await browser.pause(500);
        await expectTraceOutput(browser, receiverPlayer, [
            "received:test:0 args",
        ]);

        // Clean up
        await disconnectReceiver(browser, receiverPlayer);
        await expectTraceOutput(browser, receiverPlayer, ["disconnected"]);
    });

    it("connect ignores stale remote listener", async () => {
        await switchToTab(browser, tabB);
        // Inject a stale timestamp (20 seconds old) for "staleChannel"
        await browser.execute(() => {
            window.localStorage.setItem(
                "__ruffle_lc:localhost:staleChannel",
                (Date.now() - 20000).toString(),
            );
        });

        // Sender connects to "staleChannel" (which it shouldn't fail if it thought it was alive)
        const connectResult = await connectSender(
            browser,
            senderPlayer,
            "staleChannel",
        );
        expect(connectResult).to.equal("ok");
        await expectTraceOutput(browser, senderPlayer, [
            "connected:staleChannel",
        ]);

        // Clean up
        await disconnectReceiver(browser, senderPlayer);
        await expectTraceOutput(browser, senderPlayer, ["disconnected"]);
    });

    it("send ignores stale remote listener", async () => {
        await switchToTab(browser, tabB);
        // Inject a stale timestamp (20 seconds old) for "staleChannel2"
        await browser.execute(() => {
            window.localStorage.setItem(
                "__ruffle_lc:localhost:staleChannel2",
                (Date.now() - 20000).toString(),
            );
        });

        // Sender sends to "staleChannel2". It should fail with status error because the listener is considered dead.
        await sendMessage(browser, senderPlayer, "staleChannel2", "test");
        await expectTraceOutput(browser, senderPlayer, [
            "sent:staleChannel2:test",
        ]);
        await expectTraceOutput(browser, senderPlayer, ["status:error"]);
    });

    it("drops messages over 40KB", async () => {
        await switchToTab(browser, tabA);
        const result = await connectReceiver(
            browser,
            receiverPlayer,
            "hugeChannel",
        );
        expect(result).to.equal("ok");
        await expectTraceOutput(browser, receiverPlayer, [
            "connected:hugeChannel",
        ]);

        await switchToTab(browser, tabB);
        // Generate a 41KB string
        const hugeString = "A".repeat(41 * 1024);

        await sendMessage(
            browser,
            senderPlayer,
            "hugeChannel",
            "test",
            hugeString,
        );
        await expectTraceOutput(browser, senderPlayer, [
            "sent:hugeChannel:test",
        ]);
        // Expect an error status because it exceeds the 40KB limit
        await expectTraceOutput(browser, senderPlayer, ["status:error"]);

        // Clean up
        await switchToTab(browser, tabA);
        await disconnectReceiver(browser, receiverPlayer);
        await expectTraceOutput(browser, receiverPlayer, ["disconnected"]);
    });

    it("cleans up localStorage on beforeunload", async () => {
        // We need a pristine new tab for this test so we can safely close it.
        await browser.newWindow(
            "http://localhost:4567/test_assets/js_api.html",
        );
        await injectRuffleAndWait(browser);
        const tempPlayer = await createPlayer(
            browser,
            "/test/integration_tests/local_connection_cross_tab/receiver.swf",
        );
        const handles = await browser.getWindowHandles();
        const tempTab = handles[handles.length - 1]!;

        await switchToTab(browser, tempTab);
        const result = await connectReceiver(
            browser,
            tempPlayer,
            "unloadChannel",
        );
        expect(result).to.equal("ok");
        await expectTraceOutput(browser, tempPlayer, [
            "connected:unloadChannel",
        ]);

        // Verify it was added to localStorage
        const hasKey = await browser.execute(() => {
            return (
                window.localStorage.getItem(
                    "__ruffle_lc:localhost:unloadchannel",
                ) !== null
            );
        });
        expect(hasKey).to.equal(true);

        // Navigate away, which guarantees beforeunload/unload are fired
        await browser.url("about:blank");

        // Switch back to tab A to check localStorage
        await switchToTab(browser, tabA);
        // Wait for the navigated tab's unload handlers to finish executing
        await browser.pause(500);

        // Verify it was removed from localStorage by the navigated tab's beforeunload handler
        const hasKeyAfter = await browser.execute(() => {
            return (
                window.localStorage.getItem(
                    "__ruffle_lc:localhost:unloadchannel",
                ) !== null
            );
        });
        expect(hasKeyAfter).to.equal(false);
    });
});
