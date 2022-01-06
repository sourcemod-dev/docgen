import axios from 'axios';
import { Bundle, IBundle } from '../src';

test('test', async () => {
    const data: IBundle = (await axios.get('https://raw.githubusercontent.com/sourcemod-dev/manifest/bundles/core.bundle'))
        .data;

    const bundle = new Bundle(data);

    const ret = (await bundle.search('ArrayList', { parents: [] }))
        .filter(e => e.score > 0.5)
        .sort((a, b) => b.score - a.score);

    console.log(ret);
})
