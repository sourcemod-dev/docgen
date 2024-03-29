import { Meta } from './meta';
import { Source } from './manifest';

export * from './symbol';
export * from './bundle';
export * from './meta';
export * from './manifest';
export * from './searchable';
export * from './metadata';

export interface Index {
    /**
     * Meta descriptor of manifest content
     */
    meta: Meta;

    /**
     * Meta content source
     */
    source: Source;

    /**
     * Bundle file file stem (excluding ext)
     */
    file_stem: string,
}
