/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package ns {

    public class A {
        private namespace Key
        private var friends = [ B ]
        function beMyFriend( suitor ) {
        for each( friend in friends )
        {
            if( suitor is friend ) return Key
        }
        return null
        }
        Key function makeMyDay()
        {
        return "making my day";
        }
    }

}
