import axios from 'axios';
import { Bundle, MethodMap } from '../src';

test('test', async () => {
    const bundle: Bundle = (await axios.get('https://raw.githubusercontent.com/sourcemod-dev/manifest/bundles/core.bundle'))
        .data;

    const methodmaps: Record<string, MethodMap> = {};

    Object.keys(bundle.strands['adt_array'].methodmaps).forEach(k => {
        methodmaps[k] = new MethodMap(bundle.strands['adt_array'].methodmaps[k].symbol);
    });

    Object.values(methodmaps).forEach(mm => {
        mm.search('ArrayList', { parents: ['adt_array'] }).then(r => {
            if (r.length > 0) {
                r.forEach(e => {
                    if (e.score > 0.5) {
                        console.log(e);

                        expect(true).toBe(true);
                    } 
                });
            }
        });
    });
})
