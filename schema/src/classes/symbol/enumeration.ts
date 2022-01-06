import { Declaration } from './base';
import { IEnumeration, IEntry, Searchable, Identifier } from '../../interfaces';

export class Enumeration extends Declaration implements IEnumeration, Searchable {
    /**
     * @brief Enum entries
     * @readonly
     */
    readonly entries: IEntry[];

    readonly identifier: Identifier = Identifier.Enumeration;

    public constructor(enumeration: IEnumeration) {
        super(enumeration);

        this.entries = enumeration.entries;
    }

    // TODO: Implement search
}
