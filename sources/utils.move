module htf::utils {
  use iota::vec_map::{VecMap, Self};

  public(package) fun contains_one_of<D : copy + drop>(source : &vector<D>, one_of : &vector<D>)  : bool {
    let len_one_of = vector::length<D>(one_of);
    let mut idx_one_of = 0;
    while  ( idx_one_of < len_one_of )  {

      if (vector::contains<D>(source, &one_of[idx_one_of]) ) {
        return true
      };
      idx_one_of = idx_one_of + 1;

    };
    return true
  }

  public(package) fun contains_all_from<D : copy + drop>(source : &vector<D>, all_from : &vector<D>) : bool {
    // if encounter ANY mistake, return false
    let len_all_from = all_from.length();
    let mut idx_all_from = 0;
    while (idx_all_from < len_all_from ) {
      if (! source.contains(&all_from[idx_all_from])  ) {
        return false
      };
      idx_all_from  = idx_all_from + 1;
    };
    return true
  }


  public(package) fun copy_vector<D: copy>(src : &vector<D>) : vector<D>  {
    let mut idx = 0 ;
    let mut cloned  : vector<D>  = vector::empty();
    while (idx < src.length()) {
      cloned.push_back(src[idx]);
      idx = idx + 1;
    };
    cloned
  }
}
