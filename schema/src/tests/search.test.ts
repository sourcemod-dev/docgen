import axios from 'axios';
import { Bundle, IBundle, SearchResult } from '..';

test('test', async () => {
    const data: IBundle = (await axios.get('https://raw.githubusercontent.com/sourcemod-dev/manifest/bundles/core.bundle'))
        .data;

    console.time('Parsing bundle');

    const bundle = new Bundle(data);

    console.timeEnd('Parsing bundle');

    console.time('Searching bundle');

    const ret = (await bundle.search('ArrayList', { parents: [] }))
        .sort((a: SearchResult, b: SearchResult) => b.score - a.score);

    console.timeEnd('Searching bundle');

    console.log('Looking up symbol by path:', ret[1].path);

    const l = bundle.getSymbolByPath(ret[1].path);

    console.log(l);
})
