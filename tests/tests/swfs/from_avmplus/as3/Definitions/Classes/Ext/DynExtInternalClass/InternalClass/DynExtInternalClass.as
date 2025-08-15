/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */



package InternalClass {

    import InternalClass.*;

    // PUBLIC wrapper function for the dynamic class to be accessed;
    // otherwise it will give the error:
    // ReferenceError: DynExtInternalClass is not defined
        //  at global$init()
    public class DynExtInternalClass extends DynExtInternalClassInner  {}

}
