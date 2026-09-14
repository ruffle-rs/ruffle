package {

import flash.display.Sprite;

public class Test extends Sprite {
    public function Test() {
        var stringJsons = [
            null,
            undefined,
            '',
            ' ',
            '?',
            '-',
            '=',
            '+',
            'x',
            '{}',
            '{',
            '}',
            '{}}',
            '{}{}',
            'null',
            '{null}',
            '[]',
            '[',
            ']',
            '[]]',
            '[null]',
            'undefined',
            'Null',
            'NULL'
        ];

        for each (var stringJson in stringJsons) {
            try {
                var parsed = JSON.parse(stringJson);
                trace(stringJson + " -> " + parsed);
            } catch (e) {
                trace(stringJson + " -> threw " + e.getStackTrace());
            }
        }
    }
}

}
