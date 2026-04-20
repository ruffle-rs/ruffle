/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package OverrideFunctionBody {

    public class TestObjBase {
        public function noReturnNoParams() { return null; }
        public function noReturnParams(s:String, b:Boolean) { return null; }
        public function noReturnCustomParam(c:Custom) { return null; }
        public function returnNoParams():String { return null; }
        public function returnParams(s:String, b:Boolean):String { return null; }
        public function returnCustomNoParams():Custom { return null; }
    }

}
