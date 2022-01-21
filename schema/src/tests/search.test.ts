import axios from 'axios';
import { Bundle, IBundle, SearchResult } from '..';

test('test', async () => {
    const data: IBundle = (await axios.get('https://gist.githubusercontent.com/rumblefrog/e1b25177b8d4df230bf1fd61dc8e71bf/raw/e3cf9ae618082f78179f80ddb9e9b5d0d22ffa50/t.bundle'))
        .data;

    console.time('Parsing bundle');

    const bundle = new Bundle(data);

    console.timeEnd('Parsing bundle');

    console.time('Searching bundle');

    const ret = (await bundle.search('ArrayList', { parents: [] }))
        .sort((a: SearchResult, b: SearchResult) => b.score - a.score);

    console.timeEnd('Searching bundle');

    const l = bundle.getSymbolByPath(ret[1].path);

    console.log(l);

    // console.log(ret);
})
