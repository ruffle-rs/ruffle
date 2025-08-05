/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package voidEvaluation {

    public class TestObj
    {
        public var varF:String = "not set";
        public var varG:String = "not set";

        public function f():void
        {
            varF = "hello from f";
        }

        public function g():void
        {
            varG = "hello from g";
            return f();
        }
    }

}
