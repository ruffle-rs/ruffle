package {
import flash.display.Sprite;
import flash.events.TimerEvent;
import flash.utils.Timer;
import flash.utils.clearInterval;
import flash.utils.clearTimeout;
import flash.utils.getTimer;
import flash.utils.setInterval;
import flash.utils.setTimeout;

public class Test extends Sprite {
    // Keep the number of timers created below 127: with more, Flash Player was
    // observed to occasionally run timers due at the same time out of creation
    // order.
    private static const DELAY: int = 50;
    // Far enough apart that a pause (e.g. GC) does not merge groups of delays.
    private static const GROUP_GAP: int = 250;
    private static const CASE_DURATION: int = 1000;

    private var cases: Array = [
        "same delay", sameDelay,
        "mixed delays", mixedDelays,
        "mixed kinds", mixedKinds,
        "intervals", intervals,
        "cleared", cleared,
        "restarted timer", restartedTimer,
        "reset timer", resetTimer,
        "delay changed", delayChanged,
        "created in callbacks", createdInCallbacks,
        "overdue together", overdueTogether,
        "overdue intervals", overdueIntervals
    ];

    public function Test() {
        nextCase();
    }

    private function nextCase(): void {
        if (cases.length == 0) {
            trace("Done");
            return;
        }
        trace("// " + cases.shift());
        cases.shift()();
        setTimeout(nextCase, CASE_DURATION);
    }

    private function log(name: String): void {
        trace(name);
    }

    private function startTimer(name: String): Timer {
        var timer: Timer = new Timer(DELAY, 1);
        timer.addEventListener(TimerEvent.TIMER, function(e: TimerEvent): void {
            log(name);
        });
        timer.start();
        return timer;
    }

    private function sameDelay(): void {
        for (var i: int = 0; i < 8; i++) {
            setTimeout(log, DELAY, "timeout " + i);
        }
    }

    private function mixedDelays(): void {
        var multipliers: Array = [2, 1, 3, 1, 2, 3, 1, 2];
        for (var i: int = 0; i < multipliers.length; i++) {
            setTimeout(log, GROUP_GAP * multipliers[i], "timeout " + i + " x" + multipliers[i]);
        }
    }

    private function mixedKinds(): void {
        setTimeout(log, DELAY, "setTimeout 0");
        startTimer("Timer 1");
        var id: uint = setInterval(function(): void {
            log("setInterval 2");
            clearInterval(id);
        }, DELAY);
        startTimer("Timer 3");
        setTimeout(log, DELAY, "setTimeout 4");
    }

    private function intervals(): void {
        for (var i: int = 0; i < 3; i++) {
            startInterval("interval " + i);
        }
    }

    private function startInterval(name: String): void {
        var repeat: int = 0;
        var id: uint = setInterval(function(): void {
            log(name + " repeat " + repeat);
            if (++repeat == 3) {
                clearInterval(id);
            }
        }, DELAY);
    }

    private function cleared(): void {
        var ids: Array = [];
        ids.push(setTimeout(function(): void {
            log("timeout 0, clearing 3");
            clearTimeout(ids[3]);
        }, DELAY));
        for (var i: int = 1; i < 6; i++) {
            ids.push(setTimeout(log, DELAY, "timeout " + i));
        }
        clearTimeout(ids[2]);
        clearTimeout(ids[4]);
    }

    private function restartedTimer(): void {
        var a: Timer = startTimer("Timer A");
        startTimer("Timer B");
        startTimer("Timer C");
        a.stop();
        a.start();
    }

    private function resetTimer(): void {
        var a: Timer = startTimer("Timer A");
        startTimer("Timer B");
        startTimer("Timer C");
        a.reset();
        a.start();
    }

    private function delayChanged(): void {
        var a: Timer = startTimer("Timer A");
        startTimer("Timer B");
        startTimer("Timer C");
        a.delay = DELAY;
    }

    private function createdInCallbacks(): void {
        for each (var name: String in ["P", "Q"]) {
            setTimeout(spawn, DELAY, name);
        }
    }

    private function overdueTogether(): void {
        // The gap is larger than a step of the clock, which can happen between
        // the two calls, so the 10ms timeout is always due first.
        setTimeout(log, 30, "timeout 30ms");
        setTimeout(log, 10, "timeout 10ms");
        // Make both overdue before timers are checked again, with room for
        // a step of the clock.
        var start: int = getTimer();
        while (getTimer() - start < 50) {}
    }

    private function overdueIntervals(): void {
        var seen: Object = {};
        var ids: Array = [];
        var run: Function = function(name: String): void {
            // Stop at the first repeat.
            if (seen[name]) {
                for each (var id: uint in ids) {
                    clearInterval(id);
                }
                return;
            }
            seen[name] = true;
            log(name);
        };
        ids.push(setInterval(run, 30, "interval 30ms"));
        ids.push(setInterval(run, 10, "interval 10ms"));
        // Make both overdue before timers are checked again, with room for
        // a step of the clock.
        var start: int = getTimer();
        while (getTimer() - start < 50) {}
    }

    private function spawn(name: String): void {
        log(name);
        setTimeout(log, DELAY, name + "1");
        setTimeout(log, DELAY, name + "2");
    }
}
}
