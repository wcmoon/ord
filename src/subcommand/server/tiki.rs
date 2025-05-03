use super::*;

#[derive(serde::Deserialize)]
pub(super) struct DeployQuery {
  rune: String,
  symbol: Option<String>,
  divisibility: Option<String>,
  premine: Option<String>,
  amount: Option<String>,
  cap: Option<String>,
  height_low: Option<String>,
  height_high: Option<String>,
  offset_low: Option<String>,
  offset_high: Option<String>,
}

#[derive(serde::Deserialize)]
pub(super) struct MintQuery {
  id: String,
}

#[derive(serde::Deserialize)]
pub(super) struct CommitmentQuery {
  rune: String,
}

fn parse_str<T: std::str::FromStr>(input: Option<String>) -> Option<T> {
  input.and_then(|s| s.parse().ok())
}

pub(super) async fn deploy_encode(
  Query(query): Query<DeployQuery>
) -> ServerResult {
  task::block_in_place(|| {
    // let spaced_rune = serde_json::from_str::<SpacedRune>(params.get("rune").as_deref().unwrap()).unwrap();
    let spaced_rune = crate::SpacedRune::from_str(&query.rune).unwrap();

    let binding = query.symbol;
    let symbol_str = binding.as_deref();
    let symbol_char = symbol_str.and_then(|s| s.chars().next());

    let divisibility: Option<u8> = parse_str(query.divisibility);
    let premine: Option<u128> = parse_str(query.premine);
    let amount: Option<u128> = parse_str(query.amount);
    let cap: Option<u128> = parse_str(query.cap);
    let hl: Option<u64> = parse_str(query.height_low);
    let hh: Option<u64> = parse_str(query.height_high);
    let ol: Option<u64> = parse_str(query.offset_low);
    let oh: Option<u64> = parse_str(query.offset_high);

    let runestone = Runestone {
      etching: Some(Etching {
        // divisibility: Some(query.divisibility.as_deref().unwrap().parse::<u8>().unwrap()),
        divisibility,
        premine,
        rune: Some(spaced_rune.rune),
        spacers: Some(spaced_rune.spacers),
        symbol: symbol_char,
        terms: Some(Terms {
          amount,
          cap,
          height: (hl, hh),
          offset: (ol, oh),
        }),
        turbo: true
      }),
      ..default()
    };

    Ok(Json(runestone.encipher()).into_response())
  })
}

pub(super) async fn rune_commitment(
  Query(query): Query<CommitmentQuery>
) -> ServerResult {
  let spaced_rune = crate::SpacedRune::from_str(&query.rune).unwrap();
  Ok(Json(spaced_rune.rune.commitment()).into_response())
}

pub(super) async fn mint_encode(
  Query(query): Query<MintQuery>
) -> ServerResult {
  let id = RuneId::from_str(&query.id).unwrap();
  let runestone = Runestone {
    mint: Some(id),
    ..default()
  };
  Ok(Json(runestone.encipher()).into_response())
}

pub(super) async fn rune_payload(
  Extension(index): Extension<Arc<Index>>,
  Path(txid): Path<Txid>,
) -> ServerResult {
  task::block_in_place(|| {
    let transaction = index
      .get_transaction(txid)?
      .ok_or_not_found(|| format!("transaction {txid}"))?;

    let result = Runestone::decipher(&transaction);

    Ok(Json(result).into_response())
  })
}


pub(super) async fn tiki_output(
  Extension(index): Extension<Arc<Index>>,
  Path(outpoint): Path<OutPoint>,
  AcceptJson(accept_json): AcceptJson,
) -> ServerResult {
  task::block_in_place(|| {
    let (output_info, _) = index
      .get_tiki_output_info(outpoint)?
      .ok_or_not_found(|| format!("output {outpoint}"))?;

    Ok(if accept_json {
      Json(output_info).into_response()
    } else {
      StatusCode::NOT_FOUND.into_response()
    })
  })
}

pub(super) async fn tiki_outputs(
  Extension(index): Extension<Arc<Index>>,
  AcceptJson(accept_json): AcceptJson,
  Json(outputs): Json<Vec<OutPoint>>,
) -> ServerResult {
  task::block_in_place(|| {
    Ok(if accept_json {
      let mut response = Vec::new();
      for outpoint in outputs {
        let (output_info, _) = index
          .get_tiki_output_info(outpoint)?
          .ok_or_not_found(|| format!("output {outpoint}"))?;

        response.push(output_info);
      }
      Json(response).into_response()
    } else {
      StatusCode::NOT_FOUND.into_response()
    })
  })
}
