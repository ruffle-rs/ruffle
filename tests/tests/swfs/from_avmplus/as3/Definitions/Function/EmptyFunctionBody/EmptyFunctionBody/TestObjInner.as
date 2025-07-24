/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package EmptyFunctionBody {

    // due to bug 106878, the base class definition must be first
    class TestObjInner{
        function noReturnNoParamsInner() { return "noReturnNoParams"; }
        function noReturnParamsInner(s:String, b:Boolean) { return s; }
        function noReturnCustomParamInner(c:Custom) { return new Custom(); }
        function returnNoParamsInner():String { return "returnNoParams"; }
        function returnParamsInner(s:String, b:Boolean):String { return s; }
        function returnCustomNoParamsInner():Custom { return new Custom(); }
    }

}

