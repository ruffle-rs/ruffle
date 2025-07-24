/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package StaticFunctionBody {

    class TestObjInner{
        static function noReturnNoParamsInner() { return "noReturnNoParams"; }
        static function noReturnParamsInner(s:String, b:Boolean) { return s; }
        static function noReturnCustomParamInner(c:Custom) { return new Custom(); }
        static function returnNoParamsInner():String { return "returnNoParams"; }
        static function returnParamsInner(s:String, b:Boolean):String { return s; }
        static function returnCustomNoParamsInner():Custom { return new Custom(); }
    }

}

