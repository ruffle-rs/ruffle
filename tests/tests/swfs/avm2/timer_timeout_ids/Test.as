// Each step traces the difference between the IDs returned by setTimeout
// before and after it, so +1 means the step did not use up any IDs.

package {
import flash.display.Sprite;
import flash.utils.Timer;
import flash.utils.clearInterval;
import flash.utils.clearTimeout;
import flash.utils.setInterval;
import flash.utils.setTimeout;

public class Test extends Sprite {
    private var last: uint;

    public function Test() {
        last = nextId();
        step("nothing");
        clearTimeout(setTimeout(noop, 100000));
        step("setTimeout");
        clearInterval(setInterval(noop, 100000));
        step("setInterval");
        var timer: Timer = new Timer(1000);
        step("new Timer");
        timer.start();
        step("start");
        timer.delay = 500;
        step("delay changed while running");
        timer.stop();
        step("stop");
        timer.delay = 400;
        step("delay changed while stopped");
        timer.start();
        step("start again");
        timer.reset();
        step("reset");
        timer.start();
        step("start after reset");
        timer.stop();
    }

    private function noop(): void {}

    private function nextId(): uint {
        var id: uint = setTimeout(noop, 100000);
        clearTimeout(id);
        return id;
    }

    private function step(name: String): void {
        var id: uint = nextId();
        trace(name + ": +" + (id - last));
        last = id;
    }
}
}
