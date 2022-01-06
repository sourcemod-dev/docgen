import { Meta } from "./meta";

export interface Manifest {
    /**
     *  Meta descriptor of manifest content
     */
    meta: Meta;

    /**
     * Meta content source
     */
    source: Source;
}

export enum SourceType {
    /**
     * Git SSH URL schema
     * Repository field must be populated
     */
    Git,

    /**
     * Direct HTTP accessor
     * Endpoints should be list of URL to directly access those files
     */
    Direct,
}

export interface Source {
    /**
     * Type of source or method of access
     */
    type: SourceType;

    /**
     * If true, all sources will be merged into a single namespace when consumed by the UI
     */
    merge: boolean | null;

    /**
     * Mandatory if Git is selected as the type
     */
    repository: string | null;

    /**
     * Mandatory if Direct is selected as the type
     */
    endpoints: string[] | null;

    /**
     * Used as regex glob pattern when Git is selected
     */
    patterns: string[] | null;
}
