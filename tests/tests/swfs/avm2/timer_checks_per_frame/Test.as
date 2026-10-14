package {
import flash.display.Sprite;
import flash.events.Event;
import flash.utils.clearInterval;
import flash.utils.getTimer;
import flash.utils.setInterval;

[SWF(frameRate="4")]
public class Test extends Sprite {
    // Each handler waits longer than the interval, so that the interval is
    // overdue at the end of every handler.
    private static const WAIT: int = 30;
    private static const INTERVAL: int = 10;
    private static const FRAMES: int = 5;

    private var frame: int = 0;
    private var phase: String = null;
    private var runs: Object = {};
    private var id: uint;

    public function Test() {
        stage.addEventListener(Event.ENTER_FRAME, onEnterFrame);
        stage.addEventListener(Event.FRAME_CONSTRUCTED, onFrameConstructed);
        stage.addEventListener(Event.EXIT_FRAME, onExitFrame);
        stage.addEventListener(Event.RENDER, onRender);
        id = setInterval(onInterval, INTERVAL);
    }

    private function busyWait(): void {
        var start: int = getTimer();
        while (getTimer() - start < WAIT) {}
    }

    private function onInterval(): void {
        if (phase != null) {
            runs[phase] = (runs[phase] || 0) + 1;
        }
    }

    private function count(phase: String): int {
        return runs[phase] || 0;
    }

    private function onEnterFrame(e: Event): void {
        if (frame > 0) {
            // The number of runs while idle depends on how often timers are
            // checked, so only report whether it ran several times.
            var afterRender: int = count("render");
            trace("frame " + frame
                + ": after enterFrame " + count("enterFrame")
                + ", after frameConstructed " + count("frameConstructed")
                + ", after exitFrame " + count("exitFrame")
                + ", after render " + (afterRender >= 3 ? "3 or more" : afterRender));
        }
        frame++;
        runs = {};
        if (frame > FRAMES) {
            clearInterval(id);
            stage.removeEventListener(Event.ENTER_FRAME, onEnterFrame);
            stage.removeEventListener(Event.FRAME_CONSTRUCTED, onFrameConstructed);
            stage.removeEventListener(Event.EXIT_FRAME, onExitFrame);
            stage.removeEventListener(Event.RENDER, onRender);
            trace("Done");
            return;
        }
        stage.invalidate();
        busyWait();
        phase = "enterFrame";
    }

    private function onFrameConstructed(e: Event): void {
        busyWait();
        phase = "frameConstructed";
    }

    private function onExitFrame(e: Event): void {
        busyWait();
        phase = "exitFrame";
    }

    private function onRender(e: Event): void {
        busyWait();
        phase = "render";
    }
}
}
