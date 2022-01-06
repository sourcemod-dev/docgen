import { ITypeSet, IType } from '../../interfaces';
import { Declaration } from './base';

export class TypeSet extends Declaration implements ITypeSet {
    /**
     * @brief Type signatures
     * @readonly
     */
     readonly types: IType[];

    public constructor(typeSet: ITypeSet) {
        super(typeSet);

        this.types = typeSet.types;
    }
}
