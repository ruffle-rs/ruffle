// We run all of the 'testN' functions in this file twice - once
// invoked directly from a Timer event handler, and once invoked from an 'ENTER_FRAME'
// handler. This validates that we can schedule a new timer while inside the event handler
// for another timer.

package {
	import flash.display.Stage;
	import flash.events.Event;

	public class Test {
		public function run(theStage: Stage) {
			stage = theStage
			runNext()
		}
	}
}
import flash.utils.Timer;
import flash.events.Event;
import flash.events.TimerEvent;
import flash.utils.getTimer;
import flash.display.Stage;

// Note that this SWF has its framerate set to 50fps,
// which corresponds to one frame every 1000/50 = 20 ms
// We set all of our timer delays to multiples of 20ms, to
// try to avoid triggering Flash's "catchup" behavior (where
// the time between timer events is *less thasn* the requested delay)

// This allows us to have each test assert that the time between events
// is at *least* the expected delay.

// Add new tests to this array - tests are executed from back to front.
var allTests = [test9, test8, test7, test6, test5, test4, test3, test2, test1]

// Internal helpers used to run all of our tests twice

// When 'true', directly cal lthe function for our next test.
// This will result in scheduling a new timer (via Timer.start)
// from within an existing timer's event handler. After all of the tests
// have run in this mode, we set 'runDirect = false'
//
// When 'false', we will execute the next test from an 'ENTER_FRAME' handler,
// after the current timer has finished executing.
var runDirect: Boolean = true;

// A copy of 'allTests' - we pop tests from this, and then restore it after
// the first full run (with 'runDirect = true'')
var currentTests = allTests.concat()

var stage: Stage = null

function runNext() {
	if (currentTests.length == 0) {
		// Once we've run all of our tests in 'runDirect' mode,
		// run them again in non-'runDirect' modes
		if (runDirect) {
			trace("")
			trace("Restarting tests with runDirect=false")
			runDirect = false
			currentTests = allTests.concat()
		} else {
			return
		}
	}
	var nextTest = currentTests.pop()
	if (runDirect) {
		nextTest()
	} else {
		function runNextFrame(e:*) {
			stage.removeEventListener(Event.ENTER_FRAME, runNextFrame);
			nextTest()
		}
		stage.addEventListener(Event.ENTER_FRAME, runNextFrame);
	}
}

function test1() {
	var timer = new Timer(60, 3);
	var prev = getTimer();
	var delay = new DelayCheck();
	trace("test1: timer.currentCount = " + timer.currentCount);
	timer.addEventListener(TimerEvent.TIMER, function(e) {
		delay.check(timer.delay);
		trace("test1 tick: timer.currentCount = " + timer.currentCount);
	});
	timer.addEventListener(TimerEvent.TIMER_COMPLETE, function(e) {
		trace("test1 tick: timer complete at " + timer.currentCount);
		runNext();
	});
	timer.start();
}

// Test stopping a timer
function test2() {
	var timer = new Timer(60, 5);
	var delay = new DelayCheck();
	timer.addEventListener(TimerEvent.TIMER, function(e) {
		delay.check(timer.delay);
		trace("test2 tick: timer.currentCount = " + timer.currentCount);
		if (timer.currentCount == 3) {
			timer.stop();
			runNext();
		}
	});
	timer.addEventListener(TimerEvent.TIMER_COMPLETE, function(e) {
		trace("ERROR: test2 timer should not fire TIMER_COMPLETE");
	});
	timer.start();
}

// Test resetting a timer
function test3() {
	var timer = new Timer(60, 6);
	var delay = new DelayCheck();
	var resetDone = false;
	timer.addEventListener(TimerEvent.TIMER, function(e) {
		delay.check(timer.delay);
		trace("test3 tick: timer.currentCount = " + timer.currentCount);
		if (timer.currentCount == 2 && !resetDone) {
			trace("test3 RESET");
			timer.reset();
			resetDone = true;
			timer.start();
		}
	});
	timer.addEventListener(TimerEvent.TIMER_COMPLETE, function(e) {
		trace("test3 timer done");
		runNext();
	});
	timer.start();
}

// Test increasing timer repeatCount
function test4() {
	var timer = new Timer(60, 2);
	var delay = new DelayCheck();
	timer.addEventListener(TimerEvent.TIMER, function(e) {
		delay.check(timer.delay);
		trace("test4 tick: timer.currentCount = " + timer.currentCount);
		if (timer.currentCount == 2) {
			timer.repeatCount = 5;
		}
	});
	timer.addEventListener(TimerEvent.TIMER_COMPLETE, function(e) {
		trace("test4 timer done");
		// Adjusting repeatCount when the timer has finished does *not*
		// make it continue
		timer.repeatCount = 10;
		runNext();
	});
	timer.start();
}

// Test repeatCount = 0
function test5() {
	var timer = new Timer(60, 0);
	var delay = new DelayCheck();
	timer.addEventListener(TimerEvent.TIMER, function(e) {
		delay.check(timer.delay);
		if (timer.currentCount == 5) {
			trace("test5: Reached count 5");
			timer.stop();
			runNext();
		} else if (timer.currentCount > 5) {
			trace("ERROR: test5 continued after stop");
		}
	});
	timer.start();
}

// Test finishing timer on current tick
function test6() {
	var timer = new Timer(60, 0);
	var delay = new DelayCheck();
	timer.addEventListener(TimerEvent.TIMER, function(e) {
		delay.check(timer.delay);
		trace("test6 tick: timer.currentCount = " + timer.currentCount)
		if (timer.currentCount == 5) {
			timer.repeatCount = 5;
		}
	});
	timer.addEventListener(TimerEvent.TIMER_COMPLETE, function(e) {
		trace("test6 timer done");
		runNext();
	});
	timer.start();
}

// Test changing delay
function test7() {
	var timer = new Timer(40, 5);
	var delay = new DelayCheck();
	timer.addEventListener(TimerEvent.TIMER, function(e) {
		delay.check(timer.delay);
		trace("test7 tick: timer.currentCount = " + timer.currentCount);
		if (timer.currentCount == 3 && timer.delay != 100) {
			trace("test7: Increasing delay");
			timer.delay = 100;
		}
	});
	timer.addEventListener(TimerEvent.TIMER_COMPLETE, function(e) {
		trace("test7 timer done");
		runNext();
	});
	timer.start();	
}

// Test decreasing repeatCount
function test8() {
	var timer = new Timer(40, 5);
	var delay = new DelayCheck();
	timer.addEventListener(TimerEvent.TIMER, function(e) {
		delay.check(timer.delay);
		trace("test8 tick: timer.currentCount = " + timer.currentCount);
		if (timer.currentCount == 2) {
			trace("test8: setting timer.repeatCount = 1");
			timer.repeatCount = 1;
		}
	});
	timer.addEventListener(TimerEvent.TIMER_COMPLETE, function(e) {
		trace("test8 timer done");
		runNext()
	})
	timer.start();
}

function test9() {
	var timer:Timer = new Timer(200, 2);
	var delay = new DelayCheck()
	var ticked = false
	timer.addEventListener(TimerEvent.TIMER, function(e) {
		delay.check(timer.delay)
		trace("test9 tick: timer.currentCount = " + timer.currentCount);
		ticked = true
	})
	timer.addEventListener(TimerEvent.TIMER_COMPLETE, function(e) {
		trace("ERROR: test9: Unexpected TimerEvent.TIMER_COMPLETE")
	})

	function enterFrameHandler(e:*) {
		if (ticked) {
			trace("test9: setting timer.repeatCount to 1 outside of tick event handler")
			timer.repeatCount = 1;
			stage.removeEventListener(Event.ENTER_FRAME, enterFrameHandler)
			runNext()
		}		
	}
	stage.addEventListener(Event.ENTER_FRAME, enterFrameHandler)
	timer.start()
}