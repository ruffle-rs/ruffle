package {
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.events.SampleDataEvent;
    import flash.events.TimerEvent;
    import flash.media.Sound;
    import flash.media.SoundChannel;
    import flash.media.SoundMixer;
    import flash.net.URLRequest;
    import flash.utils.ByteArray;
    import flash.utils.Timer;
    import flash.utils.setTimeout;

    public class Test extends Sprite {
        private var caseIndex:int = 0;
        private var caseFinished:Boolean = false;

        private var sound:Sound;
        private var channel:SoundChannel;

        private var eventCount:int = 0;
        private var completeCount:int = 0;

        private var watchdog:Timer;

        private var soundA:Sound;
        private var soundB:Sound;
        private var channelA:SoundChannel;
        private var channelB:SoundChannel;

        private var countA:int = 0;
        private var countB:int = 0;

        private var completeA:Boolean = false;
        private var completeB:Boolean = false;

        public function Test() {
            trace("SampleDataEvent");
            setTimeout(runNextCase, 50);
        }

        private function runNextCase():void {
            cleanup();

            caseFinished = false;
            caseIndex++;

            switch (caseIndex) {
                case 1:
                    trace("case 1: normal blocks");
                    testNormalBlocks();
                    break;

                case 2:
                    trace("case 2: varying block sizes");
                    testVaryingSizes();
                    break;

                case 3:
                    trace("case 3: short final block");
                    testShortFinalBlock();
                    break;

                case 4:
                    trace("case 4: zero samples");
                    testZeroSamples();
                    break;

                case 5:
                    trace("case 5: SoundChannel.stop");
                    testStop();
                    break;

                case 6:
                    trace("case 6: stereo");
                    testStereo();
                    break;

                case 7:
                    trace("case 7: multiple sounds");
                    testMultipleSounds();
                    break;

                case 8:
                    trace("case 8: play arguments");
                    testPlayArguments();
                    break;

                case 9:
                    trace("case 9: SoundMixer.stopAll");
                    testSoundMixerStopAll();
                    break;

                case 10:
                    trace("case 10: generated sound lifecycle");
                    testGeneratedSoundLifecycle();
                    break;

                case 11:
                    trace("case 11: generated sound limit");
                    testGeneratedSoundLimit();
                    break;

                default:
                    trace("done");
                    break;
            }
        }

        private function nextSoon(delay:int = 100):void {
            if (caseFinished) {
                return;
            }

            caseFinished = true;
            setTimeout(runNextCase, delay);
        }

        private function cleanup():void {
            stopWatchdog();

            if (channel != null) {
                try {
                    channel.stop();
                } catch (e:Error) {
                }
            }

            if (channelA != null) {
                try {
                    channelA.stop();
                } catch (e1:Error) {
                }
            }

            if (channelB != null) {
                try {
                    channelB.stop();
                } catch (e2:Error) {
                }
            }

            sound = null;
            channel = null;

            soundA = null;
            soundB = null;

            channelA = null;
            channelB = null;

            eventCount = 0;
            completeCount = 0;

            countA = 0;
            countB = 0;

            completeA = false;
            completeB = false;

            caseFinished = false;
        }

        private function startWatchdog(
            ms:int,
            label:String
        ):void {
            stopWatchdog();

            watchdog = new Timer(ms, 1);
            watchdog.addEventListener(
                TimerEvent.TIMER_COMPLETE,
                function(e:TimerEvent):void {
                    trace(label + " timeout");
                    nextSoon();
                }
            );
            watchdog.start();
        }

        private function stopWatchdog():void {
            if (watchdog != null) {
                watchdog.stop();
                watchdog = null;
            }
        }

        private function writeSilence(
            data:ByteArray,
            samples:int
        ):void {
            for (var i:int = 0; i < samples; i++) {
                data.writeFloat(0.0);
                data.writeFloat(0.0);
            }
        }

        private function writeAndTrace(
            name:String,
            e:SampleDataEvent,
            samples:int,
            eventNumber:int
        ):void {
            var beforePosition:uint = e.data.position;
            var beforeLength:uint = e.data.length;

            writeSilence(e.data, samples);

            trace(
                name +
                " event=" + eventNumber +
                " position=" + e.position +
                " samples=" + samples +
                " beforePosition=" + beforePosition +
                " beforeLength=" + beforeLength +
                " afterPosition=" + e.data.position +
                " afterLength=" + e.data.length
            );
        }

        private function onComplete(e:Event):void {
            completeCount++;

            trace(
                "soundComplete" +
                " count=" + completeCount +
                " events=" + eventCount
            );

            stopWatchdog();
            nextSoon();
        }

        // Normal 4096-sample blocks and ByteArray reuse.

        private function testNormalBlocks():void {
            sound = new Sound();

            sound.addEventListener(
                SampleDataEvent.SAMPLE_DATA,
                onNormalSampleData
            );

            channel = sound.play();

            startWatchdog(3000, "normal");
        }

        private function onNormalSampleData(
            e:SampleDataEvent
        ):void {
            eventCount++;

            writeAndTrace(
                "normal",
                e,
                4096,
                eventCount
            );

            if (eventCount == 5) {
                if (channel != null) {
                    channel.stop();
                }

                trace("normal stopped events=" + eventCount);
                nextSoon();
            }
        }

        // Different legal SampleData buffer sizes.

        private function testVaryingSizes():void {
            sound = new Sound();

            sound.addEventListener(
                SampleDataEvent.SAMPLE_DATA,
                onVaryingSampleData
            );

            channel = sound.play();

            startWatchdog(4000, "varying");
        }

        private function onVaryingSampleData(
            e:SampleDataEvent
        ):void {
            eventCount++;

            var sizes:Array = [
                2048,
                4096,
                8192,
                4096
            ];

            var samples:int = int(sizes[eventCount - 1]);

            writeAndTrace(
                "varying",
                e,
                samples,
                eventCount
            );

            if (eventCount == sizes.length) {
                if (channel != null) {
                    channel.stop();
                }

                trace("varying stopped events=" + eventCount);
                nextSoon();
            }
        }

        // Writing fewer than 2048 samples ends the dynamic stream.

        private function testShortFinalBlock():void {
            sound = new Sound();

            sound.addEventListener(
                SampleDataEvent.SAMPLE_DATA,
                onShortFinal
            );

            channel = sound.play();

            if (channel != null) {
                channel.addEventListener(
                    Event.SOUND_COMPLETE,
                    function(e:Event):void {
                        completeCount++;

                        trace(
                            "short-final complete events=" +
                            eventCount
                        );

                        stopWatchdog();
                        nextSoon();
                    }
                );
            }

            startWatchdog(4000, "short-final");
        }

        private function onShortFinal(
            e:SampleDataEvent
        ):void {
            eventCount++;

            var samples:int =
                eventCount <= 3 ? 4096 : 1024;

            writeAndTrace(
                "short-final",
                e,
                samples,
                eventCount
            );
        }

        // Writing zero samples ends the dynamic stream.

        private function testZeroSamples():void {
            sound = new Sound();

            sound.addEventListener(
                SampleDataEvent.SAMPLE_DATA,
                onZeroSampleData
            );

            channel = sound.play();

            if (channel != null) {
                channel.addEventListener(
                    Event.SOUND_COMPLETE,
                    function(e:Event):void {
                        completeCount++;

                        trace(
                            "zero complete events=" +
                            eventCount +
                            " completes=" +
                            completeCount
                        );
                    }
                );
            }

            startWatchdog(1500, "zero");

            setTimeout(
                function():void {
                    stopWatchdog();

                    trace(
                        "zero final events=" +
                        eventCount +
                        " completes=" +
                        completeCount
                    );

                    nextSoon();
                },
                700
            );
        }

        private function onZeroSampleData(
            e:SampleDataEvent
        ):void {
            eventCount++;

            writeAndTrace(
                "zero",
                e,
                0,
                eventCount
            );
        }

        // SoundChannel.stop() stops further SampleData callbacks.

        private function testStop():void {
            sound = new Sound();

            sound.addEventListener(
                SampleDataEvent.SAMPLE_DATA,
                onStopSampleData
            );

            channel = sound.play();

            setTimeout(
                function():void {
                    var eventsAtStop:int = eventCount;

                    if (channel != null) {
                        channel.stop();
                    }

                    setTimeout(
                        function():void {
                            trace(
                                "stop no-more-events=" +
                                (eventCount == eventsAtStop)
                            );

                            nextSoon();
                        },
                        500
                    );
                },
                300
            );
        }

        private function onStopSampleData(
            e:SampleDataEvent
        ):void {
            eventCount++;
            writeSilence(e.data, 4096);
        }

        // Stereo data consists of left/right float sample pairs.

        private function testStereo():void {
            sound = new Sound();

            sound.addEventListener(
                SampleDataEvent.SAMPLE_DATA,
                onStereoSampleData
            );

            channel = sound.play();

            if (channel != null) {
                channel.addEventListener(
                    Event.SOUND_COMPLETE,
                    function(e:Event):void {
                        trace(
                            "stereo complete events=" +
                            eventCount
                        );

                        stopWatchdog();
                        nextSoon();
                    }
                );
            }

            startWatchdog(4000, "stereo");
        }

        private function onStereoSampleData(
            e:SampleDataEvent
        ):void {
            eventCount++;

            var samples:int =
                eventCount <= 5 ? 2048 : 1024;

            for (var i:int = 0; i < samples; i++) {
                var left:Number = 0.0;
                var right:Number = 0.0;

                if (eventCount == 1) {
                    left = 0.5;
                } else if (eventCount == 2) {
                    right = 0.5;
                } else if (eventCount == 3) {
                    left = 0.5;
                    right = -0.5;
                }

                e.data.writeFloat(left);
                e.data.writeFloat(right);
            }

            if (samples < 2048) {
                trace(
                    "stereo final position=" +
                    e.position +
                    " samples=" +
                    samples
                );
            }
        }

        // Multiple dynamic sounds keep independent stream state.

        private function testMultipleSounds():void {
            soundA = new Sound();
            soundB = new Sound();

            soundA.addEventListener(
                SampleDataEvent.SAMPLE_DATA,
                onSampleA
            );

            soundB.addEventListener(
                SampleDataEvent.SAMPLE_DATA,
                onSampleB
            );

            channelA = soundA.play();
            channelB = soundB.play();

            if (channelA != null) {
                channelA.addEventListener(
                    Event.SOUND_COMPLETE,
                    function(e:Event):void {
                        completeA = true;
                        checkMultipleComplete();
                    }
                );
            }

            if (channelB != null) {
                channelB.addEventListener(
                    Event.SOUND_COMPLETE,
                    function(e:Event):void {
                        completeB = true;
                        checkMultipleComplete();
                    }
                );
            }

            startWatchdog(5000, "multiple");
        }

        private function onSampleA(
            e:SampleDataEvent
        ):void {
            countA++;

            var samples:int =
                countA <= 3 ? 2048 : 1024;

            writeSilence(e.data, samples);
        }

        private function onSampleB(
            e:SampleDataEvent
        ):void {
            countB++;

            var samples:int =
                countB <= 3 ? 4096 : 1024;

            writeSilence(e.data, samples);
        }

        private function checkMultipleComplete():void {
            if (!completeA || !completeB) {
                return;
            }

            stopWatchdog();

            trace(
                "multiple complete" +
                " A.events=" + countA +
                " B.events=" + countB
            );

            nextSoon();
        }

        // startTime affects SampleDataEvent.position after playback begins.
        // Callback timing before Sound.play() returns is platform-dependent.

        private function testPlayArguments():void {
            sound = new Sound();

            sound.addEventListener(
                SampleDataEvent.SAMPLE_DATA,
                onPlayArgumentsSampleData
            );

            channel = sound.play(500, 3);

            if (channel != null) {
                channel.addEventListener(
                    Event.SOUND_COMPLETE,
                    function(e:Event):void {
                        trace(
                            "play-arguments complete events=" +
                            eventCount
                        );

                        stopWatchdog();
                        nextSoon();
                    }
                );
            }

            startWatchdog(4000, "play-arguments");
        }

        private function onPlayArgumentsSampleData(
            e:SampleDataEvent
        ):void {
            eventCount++;

            var samples:int =
                eventCount <= 3 ? 4096 : 1024;

            writeSilence(e.data, samples);

            if (samples < 2048) {
                trace(
                    "play-arguments final position=" +
                    e.position +
                    " samples=" +
                    samples
                );
            }
        }

        // SoundMixer.stopAll() stops further SampleData callbacks.

        private function testSoundMixerStopAll():void {
            sound = new Sound();

            sound.addEventListener(
                SampleDataEvent.SAMPLE_DATA,
                onMixerSampleData
            );

            channel = sound.play();

            setTimeout(
                function():void {
                    var eventsAtStop:int = eventCount;

                    SoundMixer.stopAll();

                    setTimeout(
                        function():void {
                            trace(
                                "stopAll no-more-events=" +
                                (eventCount == eventsAtStop)
                            );

                            nextSoon();
                        },
                        500
                    );
                },
                300
            );
        }

        private function onMixerSampleData(
            e:SampleDataEvent
        ):void {
            eventCount++;
            writeSilence(e.data, 4096);
        }


        // Once an unloaded Sound has been used for generated audio,
        // Flash does not allow load() to be called on it.

        private function testGeneratedSoundLifecycle():void {
            var generatedSound:Sound = new Sound();

            generatedSound.addEventListener(
                SampleDataEvent.SAMPLE_DATA,
                onGeneratedLifecycleSampleData
            );

            var firstChannel:SoundChannel =
                generatedSound.play();

            var secondChannel:SoundChannel =
                generatedSound.play();

            trace(
                "generated-lifecycle same-channel=" +
                (firstChannel == secondChannel)
            );

            try {
                generatedSound.load(
                    new URLRequest("missing.mp3")
                );

                trace(
                    "generated-lifecycle load succeeded"
                );
            } catch (e:Error) {
                trace(
                    "generated-lifecycle load error=" +
                    e.errorID
                );
            }

            if (firstChannel != null) {
                firstChannel.stop();
            }

            if (secondChannel != null) {
                secondChannel.stop();
            }

            nextSoon();
        }

        private function onGeneratedLifecycleSampleData(
            e:SampleDataEvent
        ):void {
            writeSilence(e.data, 4096);
        }

        // Flash allows at most 32 simultaneously playing generated sounds.
        // Further Sound.play() calls return null.

        private function testGeneratedSoundLimit():void {
            var channels:int = 0;
            var nulls:int = 0;

            for (var i:int = 0; i < 100; i++) {
                var generatedSound:Sound = new Sound();

                generatedSound.addEventListener(
                    SampleDataEvent.SAMPLE_DATA,
                    onGeneratedLimitSampleData
                );

                var generatedChannel:SoundChannel =
                    generatedSound.play();

                if (generatedChannel != null) {
                    channels++;
                } else {
                    nulls++;
                }
            }

            trace(
                "generated-limit channels=" +
                channels +
                " nulls=" +
                nulls
            );

            SoundMixer.stopAll();
            nextSoon();
        }

        private function onGeneratedLimitSampleData(
            e:SampleDataEvent
        ):void {
            writeSilence(e.data, 4096);
        }
    }
}
