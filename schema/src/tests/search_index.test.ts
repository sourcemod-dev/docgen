import { Bundle, IBundle, Identifier, SearchOptions, SearchResult, Declaration, MIN_SCORE } from '..';

const decl = (name: string) => ({
    name,
    refLine: 1,
    docStart: 0,
    docEnd: 0,
    docs: null,
    metadata: null,
});

const fn = (name: string, returnType: string, args: [string, string][]) => ({
    ...decl(name),
    kind: 'native',
    returnType,
    arguments: args.map(([name, type]) => ({ name, type, decl: '', default: null })),
});

const sig = (returnType: string, args: [string, string][]) => ({
    returnType,
    arguments: args.map(([name, type]) => ({ name, type })),
});

const data = {
    meta: { name: 'test', description: '', author: '' },
    source: { type: 'git' },
    version: null,
    strands: {
        'adt_array.inc': {
            functions: {
                CreateArray: fn('CreateArray', 'Handle', [['blocksize', 'int'], ['startsize', 'int']]),
            },
            methodmaps: {
                ArrayList: {
                    ...decl('ArrayList'),
                    parent: 'Handle',
                    methods: {
                        ArrayList: fn('ArrayList', 'ArrayList', [['blocksize', 'int']]),
                        GetString: fn('GetString', 'int', [['index', 'int'], ['buffer', 'char[]'], ['maxlength', 'int']]),
                        PushArray: fn('PushArray', 'int', [['values', 'const any[]']]),
                    },
                    properties: {
                        Length: { ...decl('Length'), type: 'int', getter: true, setter: false },
                    },
                },
            },
            enumstructs: {},
            constants: {},
            defines: {
                _adt_array_included: decl('_adt_array_included'),
            },
            enums: {},
            typesets: {},
            typedefs: {},
        },
        'clients.inc': {
            functions: {
                GetClientName: fn('GetClientName', 'bool', [['client', 'int'], ['name', 'char[]'], ['maxlen', 'int']]),
                IsClientInGame: fn('IsClientInGame', 'bool', [['client', 'int']]),
                a: fn('a', 'void', []),
            },
            methodmaps: {},
            enumstructs: {
                Player: {
                    ...decl('Player'),
                    methods: {
                        GetName: fn('GetName', 'void', [['buffer', 'char[]']]),
                    },
                    fields: {
                        userid: { ...decl('userid'), type: 'int' },
                    },
                },
            },
            constants: {
                MAXPLAYERS: decl('MAXPLAYERS'),
            },
            defines: {
                MAX_NAME_LENGTH: decl('MAX_NAME_LENGTH'),
            },
            enums: {
                NetFlow: {
                    ...decl('NetFlow'),
                    entries: {
                        NetFlow_Outgoing: { ...decl('NetFlow_Outgoing') },
                        NetFlow_Incoming: { ...decl('NetFlow_Incoming') },
                    },
                },
            },
            typesets: {
                SQLQueryCallback: {
                    ...decl('SQLQueryCallback'),
                    types: {
                        a: { ...decl(''), type: 'function void (Database db)', parsedSignature: sig('void', [['db', 'Database']]) },
                        b: { ...decl(''), type: 'int', parsedSignature: null },
                    },
                },
            },
            typedefs: {
                ListenCB: {
                    ...decl('ListenCB'),
                    type: 'function Action (int client, const char[] command)',
                    parsedSignature: sig('Action', [['client', 'int'], ['command', 'const char[]']]),
                },
                Address: {
                    ...decl('Address'),
                    type: 'int',
                    parsedSignature: null,
                },
            },
        },
    },
} as unknown as IBundle;

const needles = [
    '', 'a', 'A', ' ', 'int', 'INT', 'void', 'char[]', 'const char[]', 'constchar[]', 'const  char[]',
    'ArrayList', 'arraylist', 'Array List', 'Array', 'GetClientName', 'getclientname', 'GetClient',
    'ClientName', 'GetClinetName', 'NetFlow', 'netflow_in', 'MAX', 'max_name', 'userid', 'Player',
    'Database', 'Action', 'Length', 'Address', 'xyzzy', 'Q',
];

// Score every entry, as search did before indexing
async function linearSearch(bundle: Bundle, needle: string, options: SearchOptions): Promise<SearchResult[]> {
    const ret: SearchResult[] = [];

    for (const [include, strand] of Object.entries(bundle.strands)) {
        for (const member of [
            strand.functions,
            strand.methodmaps,
            strand.enumstructs,
            strand.constants,
            strand.defines,
            strand.enums,
            strand.typesets,
            strand.typedefs,
        ]) {
            for (const symbol of Object.values(member) as Declaration[]) {
                ret.push(...await symbol.search(needle, { ...options, parents: [...options.parents, include] }));
            }
        }
    }

    return ret.filter(e => e.score > MIN_SCORE);
}

describe('indexed search', () => {
    const bundle = new Bundle(data);

    const optionSets: SearchOptions[] = [
        { parents: [] },
        { parents: ['core'] },
        { parents: [], weighted: false },
        { parents: [], l1Only: true },
        { parents: [], identifier: Identifier.MethodMapMethod },
    ];

    for (const options of optionSets) {
        test(`matches linear search with ${JSON.stringify(options)}`, async () => {
            for (const needle of needles) {
                expect(await bundle.search(needle, options)).toEqual(await linearSearch(bundle, needle, options));
            }
        });
    }

    test('strand search matches linear search', async () => {
        for (const needle of needles) {
            const expected = (await linearSearch(bundle, needle, { parents: [] }))
                .filter(e => e.path[0] === 'clients.inc');

            expect(await bundle.strands['clients.inc'].search(needle, { parents: ['clients.inc'] })).toEqual(expected);
        }
    });

    test('finds exact, substring and fuzzy matches', async () => {
        const names = async (needle: string) => (await bundle.search(needle, { parents: [] })).map(e => e.name);

        expect(await names('GetClientName')).toContain('GetClientName');
        expect(await names('clientname')).toContain('GetClientName');
        expect(await names('GetClinetName')).toContain('GetClientName');
        expect(await names('xyzzy')).toEqual([]);
    });

    test('results do not share paths', async () => {
        const [first] = await bundle.search('ArrayList', { parents: [] });

        first.path.push('mutated');

        const [second] = await bundle.search('ArrayList', { parents: [] });

        expect(second.path).not.toContain('mutated');
    });
});
