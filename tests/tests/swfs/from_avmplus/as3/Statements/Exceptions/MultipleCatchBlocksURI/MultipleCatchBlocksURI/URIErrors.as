/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package MultipleCatchBlocksURI
{
    public class URIErrors
    {
        var a:Number;
                                 
        public function MyArgumentError(a):Number
        {
           var k:String = decodeURI("!@#$%^&*()<>?");
           return 6;
                                                        
        }
                                 
    }
                           
}
