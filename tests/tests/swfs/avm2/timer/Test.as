package {
	public class Test {}
}
import flash.utils.Timer;
import flash.events.TimerEvent;
import flash.utils.getTimer;

// Note that this SWF has its framerate set to 60fps, to improve
// the timer resolution. This seems to help avoid cases where Flash
// runs timers too *quickly* (maybe due to some explicit 'catch-up' behavior
// when the VM is running behind?)

// Test running a timer to completion
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
		test2();
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
			test3();
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
		test4();
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
		test5();
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
			test6();
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
		test7();
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
	});
	timer.start();	
}

test1();