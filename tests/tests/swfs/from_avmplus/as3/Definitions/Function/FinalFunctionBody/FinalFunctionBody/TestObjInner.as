/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package FinalFunctionBody {

    // due to bug 106878, the base class definition must be first
    class TestObjInner{
        final function noReturnNoParamsInner() { return "noReturnNoParams"; }
        final function noReturnParamsInner(s:String, b:Boolean) { return s; }
        final function noReturnCustomParamInner(c:Custom) { return new Custom(); }
        final function returnNoParamsInner():String { return "returnNoParams"; }
        final function returnParamsInner(s:String, b:Boolean):String { return s; }
        final function returnCustomNoParamsInner():Custom { return new Custom(); }
    }

}

