import axios from 'axios';
import { Bundle, IBundle } from '../src';

test('test', async () => {
    const data: IBundle = (await axios.get('https://raw.githubusercontent.com/sourcemod-dev/manifest/bundles/core.bundle'))
        .data;

    console.time('Parsing bundle');

    const bundle = new Bundle(data);

    console.timeEnd('Parsing bundle');

    console.time('Searching bundle');

    const ret = (await bundle.search('ArrayList', { parents: ['test_file'] }))
        .filter(e => e.score > 0.5)
        .sort((a, b) => b.score - a.score);

    console.timeEnd('Searching bundle');

    // console.log(ret);
})