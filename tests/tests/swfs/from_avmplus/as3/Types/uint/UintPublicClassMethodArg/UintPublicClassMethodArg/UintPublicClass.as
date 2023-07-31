/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
// author: Michael Tilburg

package UintPublicClassMethodArg {


    public class UintPublicClass{

        public var pubProp:uint;
        public static var pubStatProp:uint;

        public function oneArg(arg:uint):uint {
            return arg;
        }

        public function twoArg(arg1:uint, arg2:uint):uint {
            return arg1+arg2;
        }

        public function threeArg(arg1:uint, arg2:uint, arg3:uint):uint {
            return arg1+arg2+arg3;
        }

        public function diffArg(arg1:uint, arg2:int, arg3:Number):uint{
            return arg1+arg2+arg3;
        }

        public function diffArg2(arg1:int, arg2:uint, arg3:Number):uint{
            return arg1+arg2+arg3;
        }

        public function diffArg3(arg1:Number, arg2:int, arg3:uint):uint{
            return arg1+arg2+arg3;
        }

        public function useProp(arg1:uint):uint {
            pubProp = 10;
            return pubProp+arg1;
        }

    }
}



