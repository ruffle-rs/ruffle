/*
    An ActionScript implementation of stlab.adobe.com's adobe.md5_t from:
        http://stlab.adobe.com/md5_8hpp-source.html

    Copyright 2005-2008 Adobe Systems Incorporated and others
    Distributed under the MIT License (see http://stlab.adobe.com/licenses.html)

    Relevant copyright information is provided below and may not be removed from this file.
    Derived from the RSA Data Security, Inc. MD5 Message-Digest Algorithm.

        MD5C.C - RSA Data Security, Inc., MD5 message-digest algorithm

        Copyright (C) 1991-2, RSA Data Security, Inc. Created 1991. All rights
        reserved.

        License to copy and use this software is granted provided that it is
        identified as the "RSA Data Security, Inc. MD5 Message-Digest Algorithm" in
        all material mentioning or referencing this software or this function.

        License is also granted to make and use derivative works provided that such
        works are identified as "derived from the RSA Data Security, Inc. MD5
        Message-Digest Algorithm" in all material mentioning or referencing the
        derived work.

        RSA Data Security, Inc. makes no representations concerning either the
        merchantability of this software or the suitability of this software for
        any particular purpose. It is provided "as is" without express or implied
        warranty of any kind.

        These notices must be retained in any copies of any part of this
        documentation and/or software.
*/
package com.adobe.SwitchBoard
{
    import com.adobe.test.Assert;
    public final class md5_t
    {
        /*************************************************************************************************/

        // private md5_t members

        private var state_m : Array = new Array(4);
        private var count_m : Array = new Array(2);
        private var buffer_m : Array = new Array(64);

        /*************************************************************************************************/

        public static function md5(input_block : Array) : Array
        {
            const m : md5_t = new md5_t;

            m.update(input_block, input_block.length);

            return m.final();
        }

        /*************************************************************************************************/

        /* Constants for MD5Transform routine. */

        private const S11 : uint = 7;
        private const S12 : uint = 12;
        private const S13 : uint = 17;
        private const S14 : uint = 22;
        private const S21 : uint = 5;
        private const S22 : uint = 9;
        private const S23 : uint = 14;
        private const S24 : uint = 20;
        private const S31 : uint = 4;
        private const S32 : uint = 11;
        private const S33 : uint = 16;
        private const S34 : uint = 23;
        private const S41 : uint = 6;
        private const S42 : uint = 10;
        private const S43 : uint = 15;
        private const S44 : uint = 21;

        /*************************************************************************************************/

        /* F, G, H and I are basic MD5 functions. */

        private function F(x : uint, y : uint, z : uint ) : uint { return (x & y) | ((~x) & z); }
        private function G(x : uint, y : uint, z : uint ) : uint { return (x & z) | (y & (~z)); }
        private function H(x : uint, y : uint, z : uint ) : uint { return (x ^ y ^ z); }
        private function I(x : uint, y : uint, z : uint ) : uint { return (y ^ (x | (~z))); }

        /*************************************************************************************************/

        /* ROTATE_LEFT rotates x left n bits. */

        private function ROTATE_LEFT(x : uint, n : uint) : uint
        {
            return (x<<n) | (x>>>(32-n));
        }

        /*************************************************************************************************/

        /* FF, GG, HH, and II transformations for rounds 1, 2, 3, and 4.
        Rotation is separate from addition to prevent recomputation. */

        private function FF(a : uint, b : uint, c : uint, d : uint, x : uint, s : uint, ac : uint) : uint
        {
            a += F( b, c, d ) + x + ac;
            a = ROTATE_LEFT(a, s);
            a += b;
            return a;
        };

        private function GG(a : uint, b : uint, c : uint, d : uint, x : uint, s : uint, ac : uint) : uint
        {
            a += G( b, c, d ) + x + ac;
            a = ROTATE_LEFT(a, s);
            a += b;
            return a;
        };

        private function HH(a : uint, b : uint, c : uint, d : uint, x : uint, s : uint, ac : uint) : uint
        {
            a += H( b, c, d ) + x + ac;
            a = ROTATE_LEFT(a, s);
            a += b;
            return a;
        };

        private function II(a : uint, b : uint, c : uint, d : uint, x : uint, s : uint, ac : uint) : uint
        {
            a += I( b, c, d ) + x + ac;
            a = ROTATE_LEFT(a, s);
            a += b;
            return a;
        };

        /*************************************************************************************************/

        /* MD5 basic transformation. Transforms state based on block. */

        private function MD5Transform(  state : Array,
                                        block : Array ) : Array
        {
            var a : uint = state[0];
            var b : uint = state[1];
            var c : uint = state[2];
            var d : uint = state[3];
            const x : Array = Decode (block, 64);

            /* Round 1 */
            a = FF (a, b, c, d, x[ 0], S11, 0xd76aa478); /* 1 */
            d = FF (d, a, b, c, x[ 1], S12, 0xe8c7b756); /* 2 */
            c = FF (c, d, a, b, x[ 2], S13, 0x242070db); /* 3 */
            b = FF (b, c, d, a, x[ 3], S14, 0xc1bdceee); /* 4 */
            a = FF (a, b, c, d, x[ 4], S11, 0xf57c0faf); /* 5 */
            d = FF (d, a, b, c, x[ 5], S12, 0x4787c62a); /* 6 */
            c = FF (c, d, a, b, x[ 6], S13, 0xa8304613); /* 7 */
            b = FF (b, c, d, a, x[ 7], S14, 0xfd469501); /* 8 */
            a = FF (a, b, c, d, x[ 8], S11, 0x698098d8); /* 9 */
            d = FF (d, a, b, c, x[ 9], S12, 0x8b44f7af); /* 10 */
            c = FF (c, d, a, b, x[10], S13, 0xffff5bb1); /* 11 */
            b = FF (b, c, d, a, x[11], S14, 0x895cd7be); /* 12 */
            a = FF (a, b, c, d, x[12], S11, 0x6b901122); /* 13 */
            d = FF (d, a, b, c, x[13], S12, 0xfd987193); /* 14 */
            c = FF (c, d, a, b, x[14], S13, 0xa679438e); /* 15 */
            b = FF (b, c, d, a, x[15], S14, 0x49b40821); /* 16 */

            /* Round 2 */
            a = GG (a, b, c, d, x[ 1], S21, 0xf61e2562); /* 17 */
            d = GG (d, a, b, c, x[ 6], S22, 0xc040b340); /* 18 */
            c = GG (c, d, a, b, x[11], S23, 0x265e5a51); /* 19 */
            b = GG (b, c, d, a, x[ 0], S24, 0xe9b6c7aa); /* 20 */
            a = GG (a, b, c, d, x[ 5], S21, 0xd62f105d); /* 21 */
            d = GG (d, a, b, c, x[10], S22,  0x2441453); /* 22 */
            c = GG (c, d, a, b, x[15], S23, 0xd8a1e681); /* 23 */
            b = GG (b, c, d, a, x[ 4], S24, 0xe7d3fbc8); /* 24 */
            a = GG (a, b, c, d, x[ 9], S21, 0x21e1cde6); /* 25 */
            d = GG (d, a, b, c, x[14], S22, 0xc33707d6); /* 26 */
            c = GG (c, d, a, b, x[ 3], S23, 0xf4d50d87); /* 27 */
            b = GG (b, c, d, a, x[ 8], S24, 0x455a14ed); /* 28 */
            a = GG (a, b, c, d, x[13], S21, 0xa9e3e905); /* 29 */
            d = GG (d, a, b, c, x[ 2], S22, 0xfcefa3f8); /* 30 */
            c = GG (c, d, a, b, x[ 7], S23, 0x676f02d9); /* 31 */
            b = GG (b, c, d, a, x[12], S24, 0x8d2a4c8a); /* 32 */

            /* Round 3 */
            a = HH (a, b, c, d, x[ 5], S31, 0xfffa3942); /* 33 */
            d = HH (d, a, b, c, x[ 8], S32, 0x8771f681); /* 34 */
            c = HH (c, d, a, b, x[11], S33, 0x6d9d6122); /* 35 */
            b = HH (b, c, d, a, x[14], S34, 0xfde5380c); /* 36 */
            a = HH (a, b, c, d, x[ 1], S31, 0xa4beea44); /* 37 */
            d = HH (d, a, b, c, x[ 4], S32, 0x4bdecfa9); /* 38 */
            c = HH (c, d, a, b, x[ 7], S33, 0xf6bb4b60); /* 39 */
            b = HH (b, c, d, a, x[10], S34, 0xbebfbc70); /* 40 */
            a = HH (a, b, c, d, x[13], S31, 0x289b7ec6); /* 41 */
            d = HH (d, a, b, c, x[ 0], S32, 0xeaa127fa); /* 42 */
            c = HH (c, d, a, b, x[ 3], S33, 0xd4ef3085); /* 43 */
            b = HH (b, c, d, a, x[ 6], S34,  0x4881d05); /* 44 */
            a = HH (a, b, c, d, x[ 9], S31, 0xd9d4d039); /* 45 */
            d = HH (d, a, b, c, x[12], S32, 0xe6db99e5); /* 46 */
            c = HH (c, d, a, b, x[15], S33, 0x1fa27cf8); /* 47 */
            b = HH (b, c, d, a, x[ 2], S34, 0xc4ac5665); /* 48 */

            /* Round 4 */
            a = II (a, b, c, d, x[ 0], S41, 0xf4292244); /* 49 */
            d = II (d, a, b, c, x[ 7], S42, 0x432aff97); /* 50 */
            c = II (c, d, a, b, x[14], S43, 0xab9423a7); /* 51 */
            b = II (b, c, d, a, x[ 5], S44, 0xfc93a039); /* 52 */
            a = II (a, b, c, d, x[12], S41, 0x655b59c3); /* 53 */
            d = II (d, a, b, c, x[ 3], S42, 0x8f0ccc92); /* 54 */
            c = II (c, d, a, b, x[10], S43, 0xffeff47d); /* 55 */
            b = II (b, c, d, a, x[ 1], S44, 0x85845dd1); /* 56 */
            a = II (a, b, c, d, x[ 8], S41, 0x6fa87e4f); /* 57 */
            d = II (d, a, b, c, x[15], S42, 0xfe2ce6e0); /* 58 */
            c = II (c, d, a, b, x[ 6], S43, 0xa3014314); /* 59 */
            b = II (b, c, d, a, x[13], S44, 0x4e0811a1); /* 60 */
            a = II (a, b, c, d, x[ 4], S41, 0xf7537e82); /* 61 */
            d = II (d, a, b, c, x[11], S42, 0xbd3af235); /* 62 */
            c = II (c, d, a, b, x[ 2], S43, 0x2ad7d2bb); /* 63 */
            b = II (b, c, d, a, x[ 9], S44, 0xeb86d391); /* 64 */

            state[0] += a;
            state[1] += b;
            state[2] += c;
            state[3] += d;

            return state;
        }

        /*************************************************************************************************/

        /* Encodes input into output. Assumes len is a multiple of 4. */
        private function Encode (input : Array,
                                len : uint ) : Array
        {
            var output : Array = new Array( len );
            var i : uint = 0;
            var j : uint = 0;

            for (i = 0, j = 0; j < len; i++, j += 4)
            {
                output[j] = (input[i] & 0xff);
                output[j+1] = ((input[i] >> 8) & 0xff);
                output[j+2] = ((input[i] >> 16) & 0xff);
                output[j+3] = ((input[i] >> 24) & 0xff);
            }

            return output;
        }

        /*************************************************************************************************/

        /* Decodes input into output. Assumes len is a multiple of 4. */

        private function Decode ( input : Array,
                          len : uint ) : Array
        {
            var output : Array = new Array( len );
            var i : uint = 0;
            var j : uint = 0;

             for (i = 0, j = 0; j < len; i++, j += 4)
                output[i] = (input[j]) | ((input[j+1]) << 8) |
                    ((input[j+2]) << 16) | ((input[j+3]) << 24);

            return output;
        }

        /*************************************************************************************************/

        public function md5_t()
        { reset(); }

        /*************************************************************************************************/

        /* MD5 initialization. Begins an MD5 operation, writing a new context. */

        private function reset() : void
        {
            count_m[0] = 0;
            count_m[1] = 0;

            /* Load magic initialization constants. */

            state_m[0] = 0x67452301;
            state_m[1] = 0xefcdab89;
            state_m[2] = 0x98badcfe;
            state_m[3] = 0x10325476;
        }

        /*************************************************************************************************/

        /*
            MD5 block update operation. Continues an MD5 message-digest operation,
            processing another message block, and updating the context.
        */

        private function update(input_block : Array, input_length : uint) : void
        {

            /* Compute number of bytes mod 64 */
            var index : uint = ((count_m[0] >> 3) & 0x3f);

            /* Update number of bits */
            const lsb_length : uint = (input_length << 3); // low order length in bits
            count_m[0] += lsb_length;
            count_m[1] += count_m[0] < lsb_length; // add cary bit
            count_m[1] += (input_length >> 29); // high order bits.

            const partLen : uint =  (64 - index);

            /* Transform as many times as possible. */
            var i : uint = (0);

            if (input_length >= partLen)
            {
                for( i = 0; i < partLen; ++i )
                    buffer_m[index + i] = input_block[i];

                state_m = MD5Transform (state_m, buffer_m);

                for (i = partLen; i + 63 < input_length; i += 64)
                    state_m = MD5Transform (state_m, input_block.slice( i, i + 64 ) );

                index = 0;
             }

            /* Buffer remaining input */
            for( var k : uint = 0; k < input_length - i; ++k )
                buffer_m[index + k] = input_block[i + k];

        }

        /*************************************************************************************************/

        /*
            MD5 finalization. Ends an MD5 message-digest operation,
            writing the the message digest and resets the context.
        */

        private function final() : Array
        {
            const padding_s : Array =
            [
                0x80, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
            ];

            /* Save number of bits */
            const bits : Array = Encode (count_m, 8);

            /* Pad out to 56 mod 64. */
            const index : uint = ((count_m[0] >> 3) & 0x3f);
            const padLen : uint = ((index < 56) ? (56 - index) : (120 - index));

            update (padding_s, padLen);

            /* Append length (before padding) */
            update (bits, 8);

            /* Store state in digest */
            const digest : Array = Encode ( state_m, 16);

            /* Reset sensitive information. */
            reset();

            return digest;
        }

        /*************************************************************************************************/

        // md5String() returns the md5 digest of a string.

        public static function md5String(input_string : String) : String
        {
            // convert String to uint Array
            var i : uint;
            var input_block : Array = new Array( input_string.length );
            for( i = 0; i < input_string.length; ++i )
                input_block[i] = input_string.charCodeAt(i);

            const digest : Array = md5_t.md5( input_block );

            // convert uint Array to String
            var digest_string : String = "";
            for( i = 0; i < digest.length; ++i )
                digest_string += uintToHex( digest[i] );

            return digest_string;
        }

        /*************************************************************************************************/

        // uintToHex() returns a hex string for an uint.

        private static function uintToHex( n : uint ) : String
        {
            const c : Array = [ '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f' ];
            if( n > 0xFF )
                n = 0xFF;
            return c[ uint(n/16) ] + c[ uint(n%16) ];
        }

        /*************************************************************************************************/

        // unitTest() returns true if the md5 uint tests succeeded.

        public static function unitTest() : void
        {
            var ok : Boolean = true;
            var got : String;
            var expected : String;

            expected = "d41d8cd98f00b204e9800998ecf8427e";
            Assert.expectEq("md5_t.md5String('')", expected, md5_t.md5String(""));

            expected = "0cc175b9c0f1b6a831c399e269772661";
            Assert.expectEq("md5_t.md5String('a')", expected, md5_t.md5String ("a"));


            expected = "900150983cd24fb0d6963f7d28e17f72";
            Assert.expectEq("md5_t.md5String('abc')", expected, md5_t.md5String ("abc"));

            expected = "f96b697d7cb7938d525a2f31aaf161d0";
            Assert.expectEq("md5_t.md5String('message digest')", expected, md5_t.md5String ("message digest"));


            expected = "c3fcd3d76192e4007dfb496cca67e13b";
            Assert.expectEq("md5_t.md5String('abcdefghijklmnopqrstuvwxyz')", expected, md5_t.md5String ("abcdefghijklmnopqrstuvwxyz"));

            expected = "d174ab98d277d9f5a5611c2c9f419d9f";
            Assert.expectEq("md5_t.md5String('ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789')", expected, md5_t.md5String ("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789"));

            expected = "57edf4a22be3c955ac49da2e2107b67a";
            Assert.expectEq("md5_t.md5String('12345678901234567890123456789012345678901234567890123456789012345678901234567890')", expected, md5_t.md5String ("12345678901234567890123456789012345678901234567890123456789012345678901234567890"));
        }

        /*************************************************************************************************/

    }
}

